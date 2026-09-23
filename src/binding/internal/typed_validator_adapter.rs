// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private adapter for typed validators with unit context.

use super::super::BoundValidationContext;
use super::super::ExecutionError;
use super::super::ExecutionErrorKind;
use super::super::PreparedOutcome;
use super::super::PreparedValidator;
use super::super::ValidationValue;
use crate::Validator;
use crate::ViolationDraft;

/// Owns a typed validator and preserves its input type without storing a value.
pub(in crate::binding) struct TypedValidatorAdapter<T, V, M> {
    /// Typed validator invoked after the erased input is checked.
    validator: V,
    /// Mapper that creates a safe violation draft from the domain error.
    map_error: M,
    /// Type marker describing the borrowed value accepted by `validator`.
    marker: std::marker::PhantomData<fn() -> T>,
}

impl<T, V, M> TypedValidatorAdapter<T, V, M> {
    /// Creates an adapter for a specific type, validator, and error mapper.
    pub(in crate::binding) const fn new(validator: V, map_error: M) -> Self {
        Self {
            validator,
            map_error,
            marker: std::marker::PhantomData,
        }
    }
}

impl<T: 'static, V, M> PreparedValidator for TypedValidatorAdapter<T, V, M>
where
    V: Validator<T, ()> + Send + Sync + 'static,
    V::Error: Send + Sync,
    M: Fn(V::Error) -> ViolationDraft + Send + Sync + 'static,
{
    /// Validates a checked typed value and maps a domain error into a draft.
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        let typed = value
            .typed::<T>()
            .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::InputTypeMismatch))?;
        match self.validator.validate(typed, &()) {
            Ok(()) => Ok(PreparedOutcome::valid()),
            Err(error) => Ok(PreparedOutcome::invalid(vec![(self.map_error)(error)])
                .expect("a domain error maps to one violation draft")),
        }
    }
}
