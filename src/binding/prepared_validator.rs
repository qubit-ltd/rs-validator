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
/// Direct calls are a lower-level execution path. The caller must supply a
/// target matching [`Self::input_type`] and a context matching
/// [`Self::dependency_specs`] in count, order, type, and optional-value
/// representation. When dependency paths are provided, they must correspond
/// to the value slots. Constructing a [`BoundValidationContext`] does not
/// validate it against these declarations.
///
/// This layer does not interpret error paths or add a target prefix. When
/// integrated with model validation, ordinary rule errors use target-relative
/// paths; an error for a dependency should name a declared slot and use that
/// dependency's coordinate system. Model navigation and final path handling
/// belong to the integrating layer.
///
/// For configured execution, bind through a [`crate::ValidatorDescriptor`] or
/// [`crate::ValidatorRegistry`] and call [`crate::BoundValidator::validate`].
/// That boundary checks the declared shapes and outcome invariants and attaches
/// the bound rule ID to violation drafts. A direct call returns a
/// [`PreparedOutcome`] whose drafts do not yet carry the bound rule ID. Custom
/// implementations must document any further preconditions rather than assuming
/// the bound boundary has already run.
///
/// The direct-call example below uses a matching text target and no
/// dependencies.
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
    #[must_use]
    fn input_type(&self) -> InputType;

    /// Returns dependency slots accepted in execution order.
    #[must_use]
    fn dependency_specs(&self) -> &'static [DependencySpec];

    /// Validates one erased input with its ordered dependency context.
    ///
    /// Direct callers must satisfy the input and dependency declarations
    /// described by this trait. Use [`crate::BoundValidator::validate`] for
    /// the checked binding boundary; implementations are not required to
    /// duplicate all of that boundary's checks.
    ///
    /// # Parameters
    ///
    /// - `value`: Borrowed target matching [`Self::input_type`].
    /// - `context`: Borrowed dependency slots satisfying
    ///   [`Self::dependency_specs`], with corresponding paths when paths are
    ///   supplied.
    ///
    /// # Returns
    ///
    /// Returns the prepared valid or invalid outcome on successful execution.
    /// Invalid drafts have no bound rule ID yet. Skipping is recorded
    /// separately by the caller's orchestration policy as a
    /// [`crate::ValidationOutcome`].
    ///
    /// # Errors
    ///
    /// Returns an execution error when this implementation cannot process the
    /// input or context. A domain rejection is an invalid outcome rather than
    /// an execution error. Error paths are returned unchanged by this layer.
    /// An error return is not a guarantee that every direct-call precondition
    /// has been checked.
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
