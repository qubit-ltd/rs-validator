// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private adapter for typed validators with unit context.

use std::marker::PhantomData;

use super::super::BoundValidationContext;
use super::super::DependencySpec;
use super::super::ExecutionError;
use super::super::ExecutionErrorKind;
use super::super::InputType;
use super::super::PreparedOutcome;
use super::super::PreparedValidator;
use super::super::ValidationValue;
use crate::Validator;
use crate::ViolationDraft;

/// Owns a typed validator and preserves its input type without storing a value.
///
/// # Type Parameters
///
/// - `T`: Borrowed input type accepted by the validator.
/// - `V`: Validator invoked for inputs of type `T` with unit context.
/// - `M`: Mapper that converts validator errors into violation drafts.
pub(in crate::binding) struct TypedValidatorAdapter<T, V, M> {
    /// Typed validator invoked after the erased input is checked.
    validator: V,
    /// Mapper that creates a safe violation draft from the domain error.
    map_error: M,
    /// Type marker describing the borrowed value accepted by `validator`.
    marker: PhantomData<fn() -> T>,
}

impl<T, V, M> TypedValidatorAdapter<T, V, M> {
    /// Creates an adapter for a specific type, validator, and error mapper.
    ///
    /// # Parameters
    ///
    /// - `validator`: Typed validator invoked after the erased input is
    ///   checked.
    /// - `map_error`: Mapper that converts domain errors into violation drafts.
    ///
    /// # Returns
    ///
    /// An adapter that retains the validator and mapper without storing a `T`.
    #[must_use]
    #[inline]
    pub(in crate::binding) const fn new(validator: V, map_error: M) -> Self {
        Self {
            validator,
            map_error,
            marker: PhantomData,
        }
    }
}

impl<T: 'static, V, M> PreparedValidator for TypedValidatorAdapter<T, V, M>
where
    V: Validator<T, ()> + Send + Sync + 'static,
    M: Fn(V::Error) -> ViolationDraft + Send + Sync + 'static,
{
    /// Returns the concrete input type accepted by this adapter.
    ///
    /// # Returns
    ///
    /// The exact runtime type descriptor for `T`.
    #[inline]
    fn input_type(&self) -> InputType {
        InputType::of::<T>()
    }

    /// Returns that this unit-context adapter has no dependency slots.
    ///
    /// # Returns
    ///
    /// An empty slice because this adapter invokes its validator with unit
    /// context.
    #[inline]
    fn dependency_specs(&self) -> &'static [DependencySpec] {
        &[]
    }

    /// Validates a value after checking that its concrete type is `T`.
    ///
    /// A validator domain error becomes an invalid outcome containing one
    /// mapped violation. An execution error is returned when `value` does not
    /// contain the accepted type, indicating that the binding supplied a value
    /// with the wrong shape.
    ///
    /// # Parameters
    ///
    /// - `value`: Erased input expected to contain a borrowed value of type
    ///   `T`.
    /// - `context`: Prepared context; unused because this adapter requires no
    ///   dependencies.
    ///
    /// # Returns
    ///
    /// A valid outcome when validation succeeds, or an invalid outcome with
    /// one mapped violation when the domain validator rejects the value.
    ///
    /// # Errors
    ///
    /// Returns an execution error when `value` does not contain a value of
    /// type `T`.
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        let typed = value
            .typed::<T>()
            .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::InputTypeMismatch))?;
        match self.validator.validate(typed, &()) {
            Ok(()) => Ok(PreparedOutcome::valid()),
            Err(error) => Ok(PreparedOutcome::invalid(vec![(self.map_error)(error)])
                .expect("a domain error maps to one violation draft")),
        }
    }
}
