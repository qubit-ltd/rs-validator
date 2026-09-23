// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Adapter from typed text validators to the erased validation boundary.

use std::sync::Arc;

use super::PreparedValidator;
use super::internal::TextValidatorAdapter;
use crate::Validator;
use crate::ViolationDraft;

/// Prepares a text validator using a domain-error mapper.
///
/// The returned adapter rejects non-text erased inputs and converts each domain
/// error with `map_error`; callers should ensure the mapper does not retain raw
/// input in the resulting draft.
///
/// # Type Parameters
///
/// - `V`: Validator of borrowed text with unit context.
/// - `M`: Mapper from the validator error to a safe violation draft.
///
/// # Panics
///
/// This function does not panic. The prepared adapter reports an input-shape
/// error if it receives a non-text value.
#[must_use]
pub fn prepare_text_validator<V, M>(validator: V, map_error: M) -> Arc<dyn PreparedValidator>
where
    V: Validator<str, ()> + Send + Sync + 'static,
    V::Error: Send + Sync,
    M: Fn(V::Error) -> ViolationDraft + Send + Sync + 'static,
{
    Arc::new(TextValidatorAdapter::new(validator, map_error))
}
