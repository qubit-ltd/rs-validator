// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Adapter from typed text validators to the erased validation boundary.

use std::sync::Arc;

use super::BoundValidationContext;
use super::ExecutionError;
use super::PreparedOutcome;
use super::PreparedValidator;
use super::ValidationValue;
use crate::Validator;
use crate::ViolationDraft;
/// Owns a text validator and the mapper for its domain errors.
struct TextValidatorAdapter<V, M>(
    /// Typed validator invoked after the erased input is checked.
    V,
    /// Mapper which converts a domain error into a safe violation draft.
    M,
);
impl<V, M> PreparedValidator for TextValidatorAdapter<V, M>
where
    V: Validator<str, ()> + Send + Sync + 'static,
    V::Error: Send + Sync,
    M: Fn(V::Error) -> ViolationDraft + Send + Sync + 'static,
{
    /// Validates a checked text value and maps a domain error into a draft.
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        let text = value
            .as_text()
            .ok_or_else(|| ExecutionError::new(super::ExecutionErrorKind::InputTypeMismatch))?;
        match self.0.validate(text, &()) {
            Ok(()) => Ok(PreparedOutcome::Valid),
            Err(e) => Ok(PreparedOutcome::Invalid(vec![(self.1)(e)])),
        }
    }
}
/// Prepares a text validator using a domain-error mapper.
///
/// The returned adapter rejects non-text erased inputs and converts each domain
/// error with `map_error`; callers should ensure the mapper does not retain raw
/// input in the resulting draft.
#[must_use]
pub fn prepare_text_validator<V, M>(validator: V, map_error: M) -> Arc<dyn PreparedValidator>
where
    V: Validator<str, ()> + Send + Sync + 'static,
    V::Error: Send + Sync,
    M: Fn(V::Error) -> ViolationDraft + Send + Sync + 'static,
{
    Arc::new(TextValidatorAdapter(validator, map_error))
}
