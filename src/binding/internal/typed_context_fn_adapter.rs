// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private adapter that runs a typed validation closure with dependency
//! context.

use std::marker::PhantomData;

use super::super::BoundValidationContext;
use super::super::DependencySpec;
use super::super::ExecutionError;
use super::super::ExecutionErrorKind;
use super::super::InputType;
use super::super::PreparedOutcome;
use super::super::PreparedValidator;
use super::super::ValidationValue;

/// Runs a typed validation closure after checking the erased target type.
pub(in crate::binding) struct TypedContextFnAdapter<T, F> {
    /// Validation operation.
    call: F,
    /// Marks the concrete input type without owning an input value.
    input: PhantomData<fn(&T)>,
    /// Declared dependency slots for the operation.
    dependencies: &'static [DependencySpec],
}

impl<T, F> TypedContextFnAdapter<T, F> {
    /// Creates an adapter around one validation operation.
    pub(in crate::binding) const fn new(call: F, dependencies: &'static [DependencySpec]) -> Self {
        Self {
            call,
            input: PhantomData,
            dependencies,
        }
    }
}

impl<T, F> PreparedValidator for TypedContextFnAdapter<T, F>
where
    T: 'static,
    F: for<'a> Fn(&T, &BoundValidationContext<'a>) -> Result<PreparedOutcome, ExecutionError> + Send + Sync,
{
    /// Returns the concrete input type accepted by this closure.
    fn input_type(&self) -> InputType {
        InputType::of::<T>()
    }

    /// Returns the dependency slots declared for this closure.
    fn dependency_specs(&self) -> &'static [DependencySpec] {
        self.dependencies
    }

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
