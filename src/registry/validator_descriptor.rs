//! Immutable multi-signature validator descriptors.

use super::BindError;
use super::BindErrorKind;
use super::BoundValidator;
use super::ValidatorSignature;
use crate::NamedValidationArgument;

/// A definition containing one or more input/dependency signatures.
#[derive(Clone, Copy)]
pub struct ValidatorDescriptor {
    signatures: &'static [ValidatorSignature],
}

impl ValidatorDescriptor {
    /// Creates a descriptor from static signatures.
    ///
    /// Duplicate signature shapes are rejected when binding. Use
    /// [`Self::try_new`] when construction-time validation is desired.
    #[must_use]
    pub const fn new(signatures: &'static [ValidatorSignature]) -> Self {
        Self { signatures }
    }

    /// Creates a descriptor and checks for duplicate signature shapes.
    ///
    /// # Errors
    ///
    /// Returns `AmbiguousSignature` when the definition contains duplicates.
    pub fn try_new(
        signatures: &'static [ValidatorSignature],
    ) -> Result<Self, BindError> {
        let descriptor = Self::new(signatures);
        descriptor.validate_definition()?;
        Ok(descriptor)
    }

    /// Returns every supported signature in declaration order.
    #[must_use]
    pub const fn signatures(&self) -> &'static [ValidatorSignature] {
        self.signatures
    }

    /// Binds one explicitly selected signature and owns its prepared instance.
    ///
    /// # Errors
    ///
    /// Returns parameter or preparation errors, and rejects malformed
    /// descriptors.
    pub fn bind(
        &self,
        signature_index: usize,
        params: &[NamedValidationArgument<'_>],
    ) -> Result<BoundValidator, BindError> {
        self.validate_definition()?;
        let signature = self
            .signatures
            .get(signature_index)
            .copied()
            .ok_or_else(|| BindError::new(BindErrorKind::InvalidSelection))?;
        let prepared = (signature.prepare())(params)?;
        Ok(BoundValidator::new(prepared, signature, None))
    }

    /// Selects and binds the unique signature matching an input shape.
    ///
    /// # Errors
    ///
    /// Returns an ambiguity, unsupported-input, preparation, or declaration
    /// error.
    pub fn bind_for(
        &self,
        input: super::InputType,
        params: &[NamedValidationArgument<'_>],
    ) -> Result<BoundValidator, BindError> {
        self.validate_definition()?;
        let matches = self
            .signatures
            .iter()
            .copied()
            .filter(|signature| signature.input() == input)
            .collect::<Vec<_>>();
        let signature = match matches.as_slice() {
            [] => return Err(BindError::new(BindErrorKind::UnsupportedInput)),
            [signature] => *signature,
            _ => return Err(BindError::new(BindErrorKind::AmbiguousSignature)),
        };
        let prepared = (signature.prepare())(params)?;
        Ok(BoundValidator::new(prepared, signature, None))
    }

    pub(crate) fn validate_definition(&self) -> Result<(), BindError> {
        for (index, left) in self.signatures.iter().copied().enumerate() {
            if self.signatures[..index]
                .iter()
                .copied()
                .any(|right| left.same_shape(right))
            {
                return Err(BindError::new(BindErrorKind::AmbiguousSignature));
            }
            if left
                .dependencies()
                .iter()
                .any(|dependency| dependency.name().is_empty())
            {
                return Err(BindError::new(BindErrorKind::InvalidDeclaration));
            }
            for (dependency_index, dependency) in
                left.dependencies().iter().enumerate()
            {
                if left.dependencies()[..dependency_index]
                    .iter()
                    .any(|previous| previous.name() == dependency.name())
                {
                    return Err(BindError::new(
                        BindErrorKind::InvalidDeclaration,
                    )
                    .with_dependency(dependency.name()));
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for ValidatorDescriptor {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ValidatorDescriptor")
            .field("signature_count", &self.signatures.len())
            .finish()
    }
}
