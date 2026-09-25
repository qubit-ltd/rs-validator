// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Adapter from typed validators to the erased validation boundary.

use std::sync::Arc;

use super::PreparedValidator;
use super::internal::TypedValidatorAdapter;
use crate::Validator;
use crate::ViolationDraft;

/// Prepares a typed validator using a domain-error mapper.
///
/// The returned adapter rejects erased inputs whose exact concrete type differs
/// from `T` and converts each domain error with `map_error`; callers should
/// ensure the mapper does not retain raw input in the resulting draft.
///
/// # Type Parameters
///
/// - `T`: Borrowed value type expected by the validator.
/// - `V`: Validator of `T` with unit context.
/// - `M`: Mapper from the validator error to a safe violation draft.
///
/// # Panics
///
/// This function does not panic. The prepared adapter reports an input-shape
/// error if it receives a value with a different concrete type.
///
/// # Type Parameters
///
/// - `T`: Borrowed input type accepted by the validator; it must be `'static`.
/// - `V`: Thread-safe validator for `T` with unit context.
/// - `M`: Thread-safe mapper from the validator error to safe violation
///   metadata.
///
/// # Parameters
///
/// - `validator`: Validator invoked after exact type checking.
/// - `map_error`: Converts each domain error to a safe violation draft.
///
/// # Returns
///
/// A shared prepared adapter that accepts only values of type `T`.
#[must_use]
pub fn prepare_typed_validator<T: 'static, V, M>(validator: V, map_error: M) -> Arc<dyn PreparedValidator>
where
    V: Validator<T, ()> + Send + Sync + 'static,
    V::Error: Send + Sync,
    M: Fn(V::Error) -> ViolationDraft + Send + Sync + 'static,
{
    Arc::new(TypedValidatorAdapter::new(validator, map_error))
}
