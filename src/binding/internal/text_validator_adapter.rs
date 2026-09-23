// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private adapter for text validators with unit context.

use super::super::BoundValidationContext;
use super::super::ExecutionError;
use super::super::ExecutionErrorKind;
use super::super::PreparedOutcome;
use super::super::PreparedValidator;
use super::super::ValidationValue;
use crate::Validator;
use crate::ViolationDraft;

/// Owns a text validator and the mapper for its domain errors.
pub(in crate::binding) struct TextValidatorAdapter<V, M> {
    /// Typed validator invoked after the erased input is checked.
    validator: V,
    /// Mapper that creates a safe violation draft from the domain error.
    map_error: M,
}

impl<V, M> TextValidatorAdapter<V, M> {
    /// Creates an adapter from a validator and its domain error mapper.
    pub(in crate::binding) const fn new(validator: V, map_error: M) -> Self {
        Self { validator, map_error }
    }
}

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
            .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::InputTypeMismatch))?;
        match self.validator.validate(text, &()) {
            Ok(()) => Ok(PreparedOutcome::valid()),
            Err(error) => Ok(PreparedOutcome::invalid(vec![(self.map_error)(error)])
                .expect("a domain error maps to one violation draft")),
        }
    }
}
