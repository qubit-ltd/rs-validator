// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Adapter from typed validators that consume dependency context.

use std::error::Error;
use std::sync::Arc;

use super::BoundValidationContext;
use super::DependencySpec;
use super::PreparedValidator;
use super::internal::ContextualTypedValidatorAdapter;
use crate::Validator;
use crate::ViolationDraft;

/// Prepares a typed validator that reads its declared dependency slots.
///
/// The adapter passes the supplied [`BoundValidationContext`] directly to the
/// typed validator. When execution goes through [`super::BoundValidator`],
/// that boundary checks dependency count, shape, optionality, and paths before
/// invoking the adapter. The returned adapter does not validate the declared
/// dependency contract itself. Callers invoking [`PreparedValidator::validate`]
/// directly must supply a context satisfying
/// [`PreparedValidator::dependency_specs`]. Domain errors are converted with
/// `map_error`; callers must keep raw input out of the resulting draft.
///
/// # Type Parameters
///
/// - `T`: Borrowed value type expected by the validator.
/// - `V`: Context-aware validator of `T`.
/// - `E`: Its static domain error type, created and mapped within the
///   synchronous call, so it need not be `Send` or `Sync`.
/// - `M`: Mapper from the domain error to a safe violation draft.
///
/// # Panics
///
/// This function does not panic. A value with a different concrete type is
/// returned as an execution error by the prepared adapter.
///
/// # Parameters
///
/// - `dependencies`: Static dependency slots declared in execution order.
/// - `validator`: Thread-safe validator invoked with a checked value and
///   context.
/// - `map_error`: Converts a domain error to safe violation metadata.
///
/// # Returns
///
/// A shared prepared adapter for the supplied validator and mapper.
#[must_use]
pub fn prepare_contextual_typed_validator<T: 'static, V, E, M>(
    dependencies: &'static [DependencySpec],
    validator: V,
    map_error: M,
) -> Arc<dyn PreparedValidator>
where
    V: for<'a> Validator<T, BoundValidationContext<'a>, Error = E> + Send + Sync + 'static,
    E: Error + 'static,
    M: Fn(E) -> ViolationDraft + Send + Sync + 'static,
{
    Arc::new(ContextualTypedValidatorAdapter::new(validator, map_error, dependencies))
}
