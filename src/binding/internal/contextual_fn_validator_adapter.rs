// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private adapters for contextual validator closures.

use std::marker::PhantomData;

use super::super::BoundValidationContext;
use super::super::ExecutionError;
use super::super::ExecutionErrorKind;
use super::super::PreparedOutcome;
use super::super::PreparedValidator;
use super::super::ValidationValue;

/// Runs a text validation closure after checking the erased target shape.
pub(in crate::binding) struct TextContextFnAdapter<F> {
    /// Validation operation.
    call: F,
}

impl<F> TextContextFnAdapter<F> {
    /// Creates an adapter around one validation operation.
    pub(in crate::binding) const fn new(call: F) -> Self {
        Self { call }
    }
}

impl<F> PreparedValidator for TextContextFnAdapter<F>
where
    F: for<'a> Fn(&str, &BoundValidationContext<'a>) -> Result<PreparedOutcome, ExecutionError> + Send + Sync,
{
    /// Executes the closure with a checked text target.
    fn validate(
        &self,
        value: ValidationValue<'_>,
        context: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        let text = value
            .as_text()
            .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::InputTypeMismatch))?;
        (self.call)(text, context)
    }
}

/// Runs a typed validation closure after checking the erased target type.
pub(in crate::binding) struct TypedContextFnAdapter<T, F> {
    /// Validation operation.
    call: F,
    /// Marks the concrete input type without owning an input value.
    input: PhantomData<fn(&T)>,
}

impl<T, F> TypedContextFnAdapter<T, F> {
    /// Creates an adapter around one validation operation.
    pub(in crate::binding) const fn new(call: F) -> Self {
        Self {
            call,
            input: PhantomData,
        }
    }
}

impl<T, F> PreparedValidator for TypedContextFnAdapter<T, F>
where
    T: 'static,
    F: for<'a> Fn(&T, &BoundValidationContext<'a>) -> Result<PreparedOutcome, ExecutionError> + Send + Sync,
{
    /// Executes the closure with a checked typed target.
    fn validate(
        &self,
        value: ValidationValue<'_>,
        context: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        let typed = value
            .typed::<T>()
            .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::InputTypeMismatch))?;
        (self.call)(typed, context)
    }
}
