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
///
/// # Type Parameters
///
/// - `T`: Concrete input type accepted by the closure.
/// - `F`: Thread-safe closure that produces a prepared outcome or execution
///   error.
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
    ///
    /// # Parameters
    ///
    /// - `call`: Typed validation operation invoked after the erased input is
    ///   checked.
    /// - `dependencies`: Static dependency slots declared for the operation.
    ///
    /// # Returns
    ///
    /// An adapter that retains the operation and its dependency declarations.
    #[must_use]
    #[inline]
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
    ///
    /// # Returns
    ///
    /// The exact typed input shape accepted by the closure.
    #[inline]
    fn input_type(&self) -> InputType {
        InputType::of::<T>()
    }

    /// Returns the dependency slots declared for this closure.
    ///
    /// # Returns
    ///
    /// The static dependency slots in the order expected by the closure.
    #[inline]
    fn dependency_specs(&self) -> &'static [DependencySpec] {
        self.dependencies
    }

    /// Executes the closure with a checked typed target.
    ///
    /// # Parameters
    ///
    /// - `value`: Erased input that must contain a value of type `T`.
    /// - `context`: Borrowed context containing the declared dependency slots.
    ///
    /// # Returns
    ///
    /// The closure's prepared outcome when execution succeeds.
    ///
    /// # Errors
    ///
    /// Returns the closure's execution error, or an input type mismatch when
    /// `value` does not contain a value of type `T`.
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
