// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private adapter for typed validators with checked dependency context.

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

/// Owns a contextual typed validator and preserves its accepted type.
pub(in crate::binding) struct ContextualTypedValidatorAdapter<T, V, M> {
    /// Validator invoked with a checked value and dependency context.
    validator: V,
    /// Mapper that creates a safe violation draft from the domain error.
    map_error: M,
    /// Type marker describing the borrowed value accepted by `validator`.
    marker: std::marker::PhantomData<fn() -> T>,
    /// Declared dependency slots consumed by the validator.
    dependencies: &'static [DependencySpec],
}

impl<T, V, M> ContextualTypedValidatorAdapter<T, V, M> {
    /// Creates an adapter for a type, validator, and error mapper.
    pub(in crate::binding) const fn new(validator: V, map_error: M, dependencies: &'static [DependencySpec]) -> Self {
        Self {
            validator,
            map_error,
            marker: std::marker::PhantomData,
            dependencies,
        }
    }
}

impl<T: 'static, V, E, M> PreparedValidator for ContextualTypedValidatorAdapter<T, V, M>
where
    V: for<'a> Validator<T, BoundValidationContext<'a>, Error = E> + Send + Sync + 'static,
    E: Error + Send + Sync + 'static,
    M: Fn(E) -> ViolationDraft + Send + Sync + 'static,
{
    /// Returns the concrete input type accepted by this validator.
    fn input_type(&self) -> InputType {
        InputType::of::<T>()
    }

    /// Returns the dependency slots declared for this validator.
    fn dependency_specs(&self) -> &'static [DependencySpec] {
        self.dependencies
    }

    /// Validates a checked typed value with its dependency context.
    fn validate(
        &self,
        value: ValidationValue<'_>,
        context: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        let typed = value
            .typed::<T>()
            .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::InputTypeMismatch))?;
        match self.validator.validate(typed, context) {
            Ok(()) => Ok(PreparedOutcome::valid()),
            Err(error) => Ok(PreparedOutcome::invalid(vec![(self.map_error)(error)])
                .expect("a domain error maps to one violation draft")),
        }
    }
}
