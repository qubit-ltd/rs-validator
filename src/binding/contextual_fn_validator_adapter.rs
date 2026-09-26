// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Adapters for text and typed validation closures with dependency context.

use std::sync::Arc;

use super::BoundValidationContext;
use super::ExecutionError;
use super::PreparedOutcome;
use super::PreparedValidator;
use super::internal::TextContextFnAdapter;
use super::internal::TypedContextFnAdapter;

/// Prepares a text validation closure that receives its checked dependency
/// context.
///
/// The closure can report a valid input, one or more violation drafts, or an
/// execution failure. `BoundValidator` checks declared dependencies before it
/// invokes this adapter. When calling the returned `PreparedValidator`
/// directly, the closure is responsible for reading and checking its context.
///
/// Callers must keep raw rejected input out of violation drafts and execution
/// errors. Captured data must be owned and thread-safe.
///
/// # Type Parameters
///
/// - `F`: Thread-safe operation for one text input and its dependency context.
///
/// # Returns
///
/// A shared prepared validator that checks the target is text before invoking
/// `call`.
///
/// # Errors
///
/// The returned validator forwards errors from `call` unchanged. This function
/// itself does not fail.
#[must_use]
pub fn prepare_text_with_context<F>(call: F) -> Arc<dyn PreparedValidator>
where
    F: for<'a> Fn(&str, &BoundValidationContext<'a>) -> Result<PreparedOutcome, ExecutionError> + Send + Sync + 'static,
{
    Arc::new(TextContextFnAdapter::new(call))
}

/// Prepares a typed validation closure that receives its checked dependency
/// context.
///
/// The closure can report a valid input, one or more violation drafts, or an
/// execution failure. `BoundValidator` checks declared dependencies before it
/// invokes this adapter. When calling the returned `PreparedValidator`
/// directly, the closure is responsible for reading and checking its context.
///
/// Callers must keep raw rejected input out of violation drafts and execution
/// errors. Captured data must be owned and thread-safe.
///
/// # Type Parameters
///
/// - `T`: Exact input type, which must be `'static` for `TypeId`.
/// - `F`: Thread-safe operation for one typed input and its dependency context.
///
/// # Returns
///
/// A shared prepared validator that checks the target has type `T` before
/// invoking `call`.
///
/// # Errors
///
/// The returned validator forwards errors from `call` unchanged. This function
/// itself does not fail.
#[must_use]
pub fn prepare_typed_with_context<T: 'static, F>(call: F) -> Arc<dyn PreparedValidator>
where
    F: for<'a> Fn(&T, &BoundValidationContext<'a>) -> Result<PreparedOutcome, ExecutionError> + Send + Sync + 'static,
{
    Arc::new(TypedContextFnAdapter::new(call))
}
