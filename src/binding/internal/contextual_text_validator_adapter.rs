// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private adapter for text validators with checked dependency context.

use std::error::Error;

use super::super::BoundValidationContext;
use super::super::DependencySpec;
use super::super::ExecutionError;
use super::super::ExecutionErrorKind;
use super::super::InputType;
use super::super::PreparedOutcome;
use super::super::PreparedValidator;
use super::super::ValidationValue;
use crate::Validator;
use crate::ViolationDraft;

/// Owns a contextual text validator and the mapper for its domain errors.
pub(in crate::binding) struct ContextualTextValidatorAdapter<V, M> {
    /// Validator invoked with a checked text value and dependency context.
    validator: V,
    /// Mapper that creates a safe violation draft from the domain error.
    map_error: M,
    /// Declared dependency slots consumed by the validator.
    dependencies: &'static [DependencySpec],
}

impl<V, M> ContextualTextValidatorAdapter<V, M> {
    /// Creates an adapter from a validator and its domain error mapper.
    pub(in crate::binding) const fn new(validator: V, map_error: M, dependencies: &'static [DependencySpec]) -> Self {
        Self {
            validator,
            map_error,
            dependencies,
        }
    }
}

impl<V, E, M> PreparedValidator for ContextualTextValidatorAdapter<V, M>
where
    V: for<'a> Validator<str, BoundValidationContext<'a>, Error = E> + Send + Sync + 'static,
    E: Error + Send + Sync + 'static,
    M: Fn(E) -> ViolationDraft + Send + Sync + 'static,
{
    /// Returns the text shape accepted by this validator.
    fn input_type(&self) -> InputType {
        InputType::Text
    }

    /// Returns the dependency slots declared for this validator.
    fn dependency_specs(&self) -> &'static [DependencySpec] {
        self.dependencies
    }

    /// Validates checked text with its dependency context.
    fn validate(
        &self,
        value: ValidationValue<'_>,
        context: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        let text = value
            .as_text()
            .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::InputTypeMismatch))?;
        match self.validator.validate(text, context) {
            Ok(()) => Ok(PreparedOutcome::valid()),
            Err(error) => Ok(PreparedOutcome::invalid(vec![(self.map_error)(error)])
                .expect("a domain error maps to one violation draft")),
        }
    }
}
