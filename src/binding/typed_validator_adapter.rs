// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Adapter from typed validators to the erased validation boundary.

use std::sync::Arc;

use super::BoundValidationContext;
use super::ExecutionError;
use super::PreparedOutcome;
use super::PreparedValidator;
use super::ValidationValue;
use crate::Validator;
use crate::ViolationDraft;
/// Owns a typed validator and the mapper for its domain errors.
struct TypedValidatorAdapter<T, V, M>(
    /// Typed validator invoked after the erased input is checked.
    V,
    /// Mapper which converts a domain error into a safe violation draft.
    M,
    /// Function-shaped marker preserving `T` without claiming ownership.
    std::marker::PhantomData<fn() -> T>,
);
impl<T: 'static, V, M> PreparedValidator for TypedValidatorAdapter<T, V, M>
where
    V: Validator<T, ()> + Send + Sync + 'static,
    V::Error: Send + Sync,
    M: Fn(V::Error) -> ViolationDraft + Send + Sync + 'static,
{
    /// Validates a checked typed value and maps a domain error into a draft.
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        let typed = value
            .typed::<T>()
            .ok_or_else(|| ExecutionError::new(super::ExecutionErrorKind::InputTypeMismatch))?;
        match self.0.validate(typed, &()) {
            Ok(()) => Ok(PreparedOutcome::Valid),
            Err(e) => Ok(PreparedOutcome::Invalid(vec![(self.1)(e)])),
        }
    }
}
/// Prepares a typed validator using a domain-error mapper.
///
/// The returned adapter rejects erased inputs whose exact concrete type differs
/// from `T` and converts each domain error with `map_error`; callers should
/// ensure the mapper does not retain raw input in the resulting draft.
#[must_use]
pub fn prepare_typed_validator<T: 'static, V, M>(validator: V, map_error: M) -> Arc<dyn PreparedValidator>
where
    V: Validator<T, ()> + Send + Sync + 'static,
    V::Error: Send + Sync,
    M: Fn(V::Error) -> ViolationDraft + Send + Sync + 'static,
{
    Arc::new(TypedValidatorAdapter(validator, map_error, std::marker::PhantomData))
}
