// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private adapter that runs a text validation closure with dependency context.

use super::super::BoundValidationContext;
use super::super::DependencySpec;
use super::super::ExecutionError;
use super::super::ExecutionErrorKind;
use super::super::InputType;
use super::super::PreparedOutcome;
use super::super::PreparedValidator;
use super::super::ValidationValue;

/// Runs a text validation closure after checking the erased target shape.
///
/// # Type Parameters
///
/// * `F` is the closure that validates text using its bound context. To use
///   this adapter as a [`PreparedValidator`], `F` must also be `Send + Sync`.
pub(in crate::binding) struct TextContextFnAdapter<F> {
    /// Validation operation.
    call: F,
    /// Declared dependency slots for the operation.
    dependencies: &'static [DependencySpec],
}

impl<F> TextContextFnAdapter<F> {
    /// Creates an adapter around one validation operation.
    ///
    /// # Parameters
    ///
    /// * `call` is the validation closure to invoke for text values.
    /// * `dependencies` declares the dependency slots available to that
    ///   closure.
    #[must_use]
    #[inline]
    pub(in crate::binding) const fn new(call: F, dependencies: &'static [DependencySpec]) -> Self {
        Self { call, dependencies }
    }
}

impl<F> PreparedValidator for TextContextFnAdapter<F>
where
    F: for<'a> Fn(&str, &BoundValidationContext<'a>) -> Result<PreparedOutcome, ExecutionError> + Send + Sync,
{
    /// Returns the text shape accepted by this closure.
    #[inline]
    fn input_type(&self) -> InputType {
        InputType::Text
    }

    /// Returns the dependency slots declared for this closure.
    #[inline]
    fn dependency_specs(&self) -> &'static [DependencySpec] {
        self.dependencies
    }

    /// Executes the closure with a checked text target.
    ///
    /// # Parameters
    ///
    /// * `value` is the erased value to validate; it must contain text.
    /// * `context` provides the bound dependency values for this validation.
    ///
    /// # Returns
    ///
    /// The closure's prepared outcome, or an input type mismatch when `value`
    /// does not contain text, or an error returned by the closure.
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
