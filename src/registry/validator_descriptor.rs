// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable multi-signature validator descriptors.

use super::BindError;
use super::BindErrorKind;
use super::BoundValidator;
use super::ValidatorSignature;
use crate::NamedValidationArgument;
use crate::ValidatorId;

/// A definition containing one or more input/dependency signatures.
///
/// # Examples
///
/// ```
/// use std::convert::Infallible;
/// use std::sync::Arc;
///
/// use qubit_validator::prepare_text_validator;
/// use qubit_validator::BindError;
/// use qubit_validator::InputType;
/// use qubit_validator::NamedValidationArgument;
/// use qubit_validator::PreparedValidator;
/// use qubit_validator::Validator;
/// use qubit_validator::ValidatorDescriptor;
/// use qubit_validator::ValidatorSignature;
///
/// struct AcceptAll;
///
/// impl Validator<str> for AcceptAll {
///     type Error = Infallible;
///
///     fn validate(&self, _: &str, _: &()) -> Result<(), Self::Error> {
///         Ok(())
///     }
/// }
///
/// fn prepare(
///     _: &[NamedValidationArgument<'_>],
/// ) -> Result<Arc<dyn PreparedValidator>, BindError> {
///     Ok(prepare_text_validator(AcceptAll, |never| match never {}))
/// }
///
/// static SIGNATURES: &[ValidatorSignature] = &[
///     ValidatorSignature::new(InputType::Text, &[], prepare),
/// ];
/// let descriptor = ValidatorDescriptor::try_new(SIGNATURES)?;
/// assert_eq!(descriptor.signatures().len(), 1);
/// # Ok::<(), BindError>(())
/// ```
#[derive(Clone, Copy)]
pub struct ValidatorDescriptor {
    /// Supported signatures in declaration order.
    signatures: &'static [ValidatorSignature],
}

impl ValidatorDescriptor {
    /// Creates a descriptor from static signatures.
    ///
    /// Duplicate signature shapes are rejected when binding. Use
    /// [`Self::try_new`] when construction-time validation is desired.
    #[must_use]
    #[inline]
    pub const fn new(signatures: &'static [ValidatorSignature]) -> Self {
        Self { signatures }
    }

    /// Creates a descriptor and checks for duplicate signature shapes.
    ///
    /// # Errors
    ///
    /// Returns `InvalidDeclaration` when there are no signatures or a
    /// dependency name is empty or repeated. Returns `AmbiguousSignature` when
    /// two signatures accept the same input shape.
    pub fn try_new(signatures: &'static [ValidatorSignature]) -> Result<Self, BindError> {
        let descriptor = Self::new(signatures);
        descriptor.validate_definition()?;
        Ok(descriptor)
    }

    /// Returns every supported signature in declaration order.
    #[must_use]
    #[inline]
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
        rule_id: ValidatorId,
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
        Ok(BoundValidator::new(prepared, signature, rule_id))
    }

    /// Selects and binds the unique signature matching an input shape.
    ///
    /// # Errors
    ///
    /// Returns an ambiguity, unsupported-input, preparation, or declaration
    /// error.
    pub fn bind_for(
        &self,
        rule_id: ValidatorId,
        input: super::InputType,
        params: &[NamedValidationArgument<'_>],
    ) -> Result<BoundValidator, BindError> {
        self.validate_definition()?;
        self.bind_for_validated(rule_id, input, params)
    }

    /// Binds an input signature after a containing registry validated this
    /// descriptor while freezing its registrations.
    pub(crate) fn bind_for_validated(
        &self,
        rule_id: ValidatorId,
        input: super::InputType,
        params: &[NamedValidationArgument<'_>],
    ) -> Result<BoundValidator, BindError> {
        let signature = self
            .signatures
            .iter()
            .copied()
            .find(|signature| signature.input() == input)
            .ok_or_else(|| BindError::new(BindErrorKind::UnsupportedInput))?;
        let prepared = (signature.prepare())(params)?;
        Ok(BoundValidator::new(prepared, signature, rule_id))
    }

    /// Checks that the descriptor is non-empty and unambiguous.
    ///
    /// # Errors
    ///
    /// Returns a declaration error for empty signatures, duplicate input
    /// shapes, or duplicate or empty dependency names.
    pub(crate) fn validate_definition(&self) -> Result<(), BindError> {
        if self.signatures.is_empty() {
            return Err(BindError::new(BindErrorKind::InvalidDeclaration));
        }
        for (index, left) in self.signatures.iter().copied().enumerate() {
            if self.signatures[..index]
                .iter()
                .copied()
                .any(|right| left.input() == right.input())
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
            for (dependency_index, dependency) in left.dependencies().iter().enumerate() {
                if left.dependencies()[..dependency_index]
                    .iter()
                    .any(|previous| previous.name() == dependency.name())
                {
                    return Err(BindError::new(BindErrorKind::InvalidDeclaration).with_dependency(dependency.name()));
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
