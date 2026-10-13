// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Type-erased prepared validator instances.

use std::sync::Arc;

use super::ArgumentReader;
use super::BindError;
use super::BoundValidationContext;
use super::DependencySpec;
use super::ExecutionError;
use super::InputType;
use super::PreparedOutcome;
use super::ValidationValue;

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
///     fn input_type(&self) -> qubit_validator::InputType {
///         qubit_validator::InputType::Text
///     }
///
///     fn dependency_specs(&self) -> &'static [qubit_validator::DependencySpec] {
///         &[]
///     }
///
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
    /// Returns the exact erased target shape accepted by this implementation.
    fn input_type(&self) -> InputType;

    /// Returns dependency slots accepted in execution order.
    fn dependency_specs(&self) -> &'static [DependencySpec];

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
/// When binding through a descriptor or registry, the binding layer rejects
/// duplicate parameter names before calling the factory and rejects unconsumed
/// parameters after it returns. The factory's reader calls decide which
/// parameters are required and reject invalid types or ranges. Calling the
/// factory directly does not provide the binding layer's checks.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
///
/// use qubit_validator::{
///     ArgumentReader, BindError, BoundValidationContext, ExecutionError,
///     PrepareFn, PreparedOutcome, PreparedValidator, ValidationValue,
/// };
///
/// struct Accept;
/// impl PreparedValidator for Accept {
///     fn input_type(&self) -> qubit_validator::InputType {
///         qubit_validator::InputType::Text
///     }
///
///     fn dependency_specs(&self) -> &'static [qubit_validator::DependencySpec] {
///         &[]
///     }
///
///     fn validate(
///         &self,
///         _: ValidationValue<'_>,
///         _: &BoundValidationContext<'_>,
///     ) -> Result<PreparedOutcome, ExecutionError> {
///         Ok(PreparedOutcome::valid())
///     }
/// }
///
/// fn prepare(_reader: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
///     Ok(Arc::new(Accept))
/// }
///
/// let factory: PrepareFn = prepare;
/// let mut reader = ArgumentReader::new(&[])?;
/// let prepared = factory(&mut reader)?;
/// reader.finish()?;
/// assert_eq!(prepared.validate(
///     ValidationValue::Text("ready"),
///     &BoundValidationContext::new(&[]),
/// )?, PreparedOutcome::valid());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub type PrepareFn = for<'a> fn(&mut ArgumentReader<'a>) -> Result<Arc<dyn PreparedValidator>, BindError>;
