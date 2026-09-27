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
pub(in crate::binding) struct TextContextFnAdapter<F> {
    /// Validation operation.
    call: F,
    /// Declared dependency slots for the operation.
    dependencies: &'static [DependencySpec],
}

impl<F> TextContextFnAdapter<F> {
    /// Creates an adapter around one validation operation.
    pub(in crate::binding) const fn new(call: F, dependencies: &'static [DependencySpec]) -> Self {
        Self { call, dependencies }
    }
}

impl<F> PreparedValidator for TextContextFnAdapter<F>
where
    F: for<'a> Fn(&str, &BoundValidationContext<'a>) -> Result<PreparedOutcome, ExecutionError> + Send + Sync,
{
    /// Returns the text shape accepted by this closure.
    fn input_type(&self) -> InputType {
        InputType::Text
    }

    /// Returns the dependency slots declared for this closure.
    fn dependency_specs(&self) -> &'static [DependencySpec] {
        self.dependencies
    }

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
