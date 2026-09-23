// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Adapter from text validators that consume dependency context.

use std::error::Error;
use std::sync::Arc;

use super::BoundValidationContext;
use super::PreparedValidator;
use super::internal::ContextualTextValidatorAdapter;
use crate::Validator;
use crate::ViolationDraft;

/// Prepares a text validator that reads its declared dependency slots.
///
/// The adapter passes the checked [`BoundValidationContext`] directly to the
/// typed validator. [`super::BoundValidator`] checks dependency count, shape,
/// optionality, and paths before this adapter runs. Domain errors are converted
/// with `map_error`; callers must keep raw input out of the resulting draft.
///
/// # Type Parameters
///
/// - `V`: Text validator implementing the context-aware [`Validator`] contract.
/// - `E`: Its domain error type, which must be thread-safe and static.
/// - `M`: Mapper from the domain error to a safe violation draft.
///
/// # Panics
///
/// This function does not panic. A non-text erased input is returned as an
/// execution error by the prepared adapter.
#[must_use]
pub fn prepare_contextual_text_validator<V, E, M>(validator: V, map_error: M) -> Arc<dyn PreparedValidator>
where
    V: for<'a> Validator<str, BoundValidationContext<'a>, Error = E> + Send + Sync + 'static,
    E: Error + Send + Sync + 'static,
    M: Fn(E) -> ViolationDraft + Send + Sync + 'static,
{
    Arc::new(ContextualTextValidatorAdapter::new(validator, map_error))
}
