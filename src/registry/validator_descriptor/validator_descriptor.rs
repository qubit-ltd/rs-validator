// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable multi-signature validator descriptors.

use std::fmt;
use std::sync::Arc;

use super::super::ValidatorSignature;
use crate::ArgumentReader;
use crate::NamedValidationArgument;
use crate::PreparedValidator;
use crate::ValidatorId;
use crate::binding::BindError;
use crate::binding::BindErrorKind;
use crate::binding::BoundValidator;
use crate::binding::InputType;

/// A definition containing one or more input/dependency signatures.
///
/// # Examples
///
/// ```
/// use std::convert::Infallible;
/// use std::sync::Arc;
///
/// use qubit_validator::prepare_text_validator;
/// use qubit_validator::ArgumentReader;
/// use qubit_validator::BindError;
/// use qubit_validator::InputType;
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
///     _reader: &mut ArgumentReader<'_>,
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
    ///
    /// # Parameters
    ///
    /// - `signatures`: Static signature declarations in their selection order.
    ///
    /// # Returns
    ///
    /// A descriptor referring to the supplied declarations.
    #[must_use]
    #[inline]
    pub const fn new(signatures: &'static [ValidatorSignature]) -> Self {
        Self { signatures }
    }

    /// Creates a descriptor and checks for duplicate signature shapes.
    ///
    /// # Parameters
    ///
    /// - `signatures`: Static signature declarations in their selection order.
    ///
    /// # Returns
    ///
    /// A descriptor containing the declarations when they are non-empty and
    /// unambiguous.
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
    ///
    /// # Returns
    ///
    /// The static signature slice used by this descriptor.
    #[must_use]
    #[inline]
    pub const fn signatures(&self) -> &'static [ValidatorSignature] {
        self.signatures
    }

    /// Binds one explicitly selected signature and owns its prepared instance.
    ///
    /// # Parameters
    ///
    /// - `rule_id`: Stable rule identifier attached to the binding, its
    ///   violations, and binding errors.
    /// - `signature_index`: Zero-based index into [`Self::signatures`] in
    ///   declaration order. Reordering signatures changes this selection.
    /// - `params`: Named configuration arguments borrowed during preparation.
    ///   The factory must own any configuration it retains for execution.
    ///
    /// # Returns
    ///
    /// A reusable binding owning a shared prepared validator instance. The
    /// returned binding does not borrow `params`.
    ///
    /// # Errors
    ///
    /// Returns `InvalidDeclaration` or `AmbiguousSignature` when the
    /// descriptor is malformed, and `InvalidSelection` when
    /// `signature_index` is out of range. Parameter binding errors identify
    /// duplicate or unconsumed arguments; factory errors and prepared-shape
    /// mismatches are also returned with `rule_id` attached.
    pub fn bind<'a>(
        &self,
        rule_id: ValidatorId,
        signature_index: usize,
        params: &'a [NamedValidationArgument<'a>],
    ) -> Result<BoundValidator, BindError> {
        self.validate_definition().map_err(|error| error.with_rule(rule_id))?;
        let signature = self
            .signatures
            .get(signature_index)
            .copied()
            .ok_or_else(|| BindError::new(BindErrorKind::InvalidSelection).with_rule(rule_id))?;
        let prepared = Self::prepare_checked(rule_id, signature, params)?;
        Ok(BoundValidator::new(prepared, signature, rule_id))
    }

    /// Selects and binds the unique signature matching an input shape.
    ///
    /// # Parameters
    ///
    /// - `rule_id`: Stable rule identifier attached to the binding, its
    ///   violations, and binding errors.
    /// - `input`: Input shape used to select the matching signature; binding
    ///   does not convert values between input shapes.
    /// - `params`: Named configuration arguments borrowed during preparation.
    ///   The factory must own any configuration it retains for execution.
    ///
    /// # Returns
    ///
    /// A reusable binding owning a shared prepared validator instance for
    /// `input`. The returned binding does not borrow `params`.
    ///
    /// # Errors
    ///
    /// Returns `InvalidDeclaration` or `AmbiguousSignature` when the
    /// descriptor is malformed, and `UnsupportedInput` when no signature
    /// accepts `input`. Parameter binding errors, factory errors, and prepared
    /// shape mismatches are returned with `rule_id` attached.
    pub fn bind_for<'a>(
        &self,
        rule_id: ValidatorId,
        input: InputType,
        params: &'a [NamedValidationArgument<'a>],
    ) -> Result<BoundValidator, BindError> {
        self.validate_definition().map_err(|error| error.with_rule(rule_id))?;
        self.bind_for_validated(rule_id, input, params)
    }

    /// Binds the unique signature for an input shape after a containing
    /// registry validated this descriptor while freezing its registrations.
    ///
    /// # Parameters
    ///
    /// - `rule_id`: Stable identifier attached to returned errors.
    /// - `input`: Input shape used to select the signature.
    /// - `params`: Named arguments borrowed while preparing the validator. The
    ///   factory must own any configuration it retains.
    ///
    /// # Returns
    ///
    /// A binding that owns the prepared validator and does not borrow `params`.
    ///
    /// # Errors
    ///
    /// Returns `UnsupportedInput` when no signature accepts `input`. Returns
    /// the preparation error or `PreparedSignatureMismatch` when preparing
    /// the selected signature fails or returns a mismatched shape. Errors
    /// carry `rule_id`.
    pub(crate) fn bind_for_validated<'a>(
        &self,
        rule_id: ValidatorId,
        input: InputType,
        params: &'a [NamedValidationArgument<'a>],
    ) -> Result<BoundValidator, BindError> {
        let signature = self
            .signatures
            .iter()
            .copied()
            .find(|signature| signature.input() == input)
            .ok_or_else(|| BindError::new(BindErrorKind::UnsupportedInput).with_rule(rule_id))?;
        let prepared = Self::prepare_checked(rule_id, signature, params)?;
        Ok(BoundValidator::new(prepared, signature, rule_id))
    }

    /// Checks that the descriptor is non-empty and unambiguous.
    ///
    /// # Returns
    ///
    /// `()` when the declarations are valid.
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

    /// Checks all parameters and the prepared shape after a signature is
    /// selected.
    ///
    /// # Parameters
    ///
    /// - `rule_id`: Stable identifier attached to returned errors.
    /// - `signature`: Declaration whose factory is being invoked.
    /// - `params`: Named arguments borrowed while preparing the validator. The
    ///   factory must own any configuration it retains.
    ///
    /// # Returns
    ///
    /// The factory's prepared validator after argument and shape validation.
    ///
    /// # Errors
    ///
    /// Returns a rule-tagged error for duplicate parameters, factory failures,
    /// unconsumed parameters, or a prepared signature mismatch.
    fn prepare_checked<'a>(
        rule_id: ValidatorId,
        signature: ValidatorSignature,
        params: &'a [NamedValidationArgument<'a>],
    ) -> Result<Arc<dyn PreparedValidator>, BindError> {
        let mut reader = ArgumentReader::new(params).map_err(|error| error.with_rule(rule_id))?;
        let prepared = (signature.prepare())(&mut reader).map_err(|error| error.with_rule(rule_id))?;
        reader.finish().map_err(|error| error.with_rule(rule_id))?;
        Self::check_prepared_shape(prepared.as_ref(), signature).map_err(|error| error.with_rule(rule_id))?;
        Ok(prepared)
    }

    /// Ensures a factory produced the input and dependency shape it declared.
    ///
    /// # Parameters
    ///
    /// - `prepared`: Factory output to validate.
    /// - `signature`: Declared shape to compare with the output.
    ///
    /// # Returns
    ///
    /// `()` when the prepared validator matches its declaration.
    ///
    /// # Errors
    ///
    /// Returns `PreparedSignatureMismatch` when either shape differs.
    fn check_prepared_shape(prepared: &dyn PreparedValidator, signature: ValidatorSignature) -> Result<(), BindError> {
        if prepared.input_type() != signature.input() || prepared.dependency_specs() != signature.dependencies() {
            return Err(BindError::new(BindErrorKind::PreparedSignatureMismatch));
        }
        Ok(())
    }
}

impl fmt::Debug for ValidatorDescriptor {
    /// Formats the number of signatures without exposing preparation details.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ValidatorDescriptor")
            .field("signature_count", &self.signatures.len())
            .finish()
    }
}
