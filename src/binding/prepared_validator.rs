// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Type-erased prepared validator instances.

use std::sync::Arc;

use super::BindError;
use super::BoundValidationContext;
use super::ExecutionError;
use super::PreparedOutcome;
use super::ValidationValue;
use crate::NamedValidationArgument;

/// A configured, immutable validator instance safe to share between calls.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
///
/// use qubit_validator::{
///     BoundValidationContext, ExecutionError, NamedValidationArgument, PreparedOutcome,
///     PreparedValidator, ValidationValue,
/// };
///
/// struct Accept;
/// impl PreparedValidator for Accept {
///     fn validate(
///         &self,
///         _: ValidationValue<'_>,
///         _: &BoundValidationContext<'_>,
///     ) -> Result<PreparedOutcome, ExecutionError> {
///         Ok(PreparedOutcome::valid())
///     }
/// }
///
/// let prepared: Arc<dyn PreparedValidator> = Arc::new(Accept);
/// let arguments: &[NamedValidationArgument<'_>] = &[];
/// let result = prepared.validate(
///     ValidationValue::Text("ready"),
///     &BoundValidationContext::new(&[]),
/// );
/// assert_eq!(result?, PreparedOutcome::valid());
/// let _ = arguments;
/// # Ok::<(), ExecutionError>(())
/// ```
pub trait PreparedValidator: Send + Sync {
    /// Validates one erased input with its ordered dependency context.
    ///
    /// # Errors
    ///
    /// Returns an execution error when the input or context cannot be
    /// processed.
    fn validate(
        &self,
        value: ValidationValue<'_>,
        context: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError>;
}

/// Constructs an owned prepared validator from declaration parameters.
///
/// The function returns a binding error when parameters are missing, unknown,
/// duplicated, or cannot be decoded by the validator implementation.
pub type PrepareFn = fn(&[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError>;
