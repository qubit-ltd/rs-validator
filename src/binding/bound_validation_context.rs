// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Ordered, shape-checked dependency values for one validation call.

use super::DependencySpec;
use super::ExecutionError;
use super::ExecutionErrorKind;
use super::ValidationPath;
use super::ValidationValue;

/// Borrowed dependency slots used during a synchronous validation call.
pub struct BoundValidationContext<'a> {
    /// Values stored in signature slot order.
    values: &'a [ValidationValue<'a>],
    /// Optional model paths stored in the same slot order as `values`.
    paths: Option<&'a [ValidationPath]>,
}

impl<'a> BoundValidationContext<'a> {
    /// Creates a context with root paths for each dependency slot.
    #[must_use]
    #[inline]
    pub fn new(values: &'a [ValidationValue<'a>]) -> Self {
        Self { values, paths: None }
    }

    /// Creates a context with model-provided relative dependency paths.
    ///
    /// # Errors
    ///
    /// Returns an adapter contract error when the two slices differ in length.
    pub fn new_with_paths(
        values: &'a [ValidationValue<'a>],
        paths: &'a [ValidationPath],
    ) -> Result<Self, ExecutionError> {
        if values.len() != paths.len() {
            return Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation));
        }
        Ok(Self {
            values,
            paths: Some(paths),
        })
    }

    /// Returns the value at an ordered dependency slot.
    ///
    /// # Errors
    ///
    /// Returns an adapter contract error for an invalid slot index.
    #[inline]
    pub fn value(&self, index: usize) -> Result<ValidationValue<'a>, ExecutionError> {
        self.values
            .get(index)
            .copied()
            .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::AdapterContractViolation))
    }

    /// Returns the path at an ordered dependency slot.
    ///
    /// # Errors
    ///
    /// Returns an adapter contract error for an invalid slot index.
    #[inline]
    pub fn dependency_path(&self, index: usize) -> Result<&ValidationPath, ExecutionError> {
        match self.paths {
            Some(paths) => paths
                .get(index)
                .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::AdapterContractViolation)),
            None if index < self.values.len() => Ok(&ROOT_PATH),
            None => Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation)),
        }
    }

    /// Downcasts a required dependency to its exact type.
    ///
    /// # Errors
    ///
    /// Returns a shape, missing-value, or slot error.
    #[inline]
    pub fn typed<T: 'static>(&self, index: usize) -> Result<&'a T, ExecutionError> {
        match self.value(index)? {
            ValidationValue::Typed(value) => value
                .downcast_ref::<T>()
                .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::DependencyTypeMismatch)),
            ValidationValue::Missing => Err(ExecutionError::new(ExecutionErrorKind::MissingRequiredDependencyValue)),
            ValidationValue::Text(_) => Err(ExecutionError::new(ExecutionErrorKind::DependencyTypeMismatch)),
        }
    }

    /// Downcasts an optional dependency, treating only `Missing` as `None`.
    ///
    /// # Returns
    ///
    /// Returns `Some` with a correctly typed present value, or `None` for the
    /// explicit missing marker.
    ///
    /// # Errors
    ///
    /// Returns a shape or slot error. A wrong concrete type is never treated
    /// as an absent optional value.
    #[inline]
    pub fn optional_typed<T: 'static>(&self, index: usize) -> Result<Option<&'a T>, ExecutionError> {
        match self.value(index)? {
            ValidationValue::Missing => Ok(None),
            ValidationValue::Typed(value) => value
                .downcast_ref::<T>()
                .map(Some)
                .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::DependencyTypeMismatch)),
            ValidationValue::Text(_) => Err(ExecutionError::new(ExecutionErrorKind::DependencyTypeMismatch)),
        }
    }

    /// Borrows a required text dependency.
    ///
    /// # Errors
    ///
    /// Returns a shape, missing-value, or slot error.
    #[inline]
    pub fn text(&self, index: usize) -> Result<&'a str, ExecutionError> {
        match self.value(index)? {
            ValidationValue::Text(value) => Ok(value),
            ValidationValue::Missing => Err(ExecutionError::new(ExecutionErrorKind::MissingRequiredDependencyValue)),
            ValidationValue::Typed(_) => Err(ExecutionError::new(ExecutionErrorKind::DependencyTypeMismatch)),
        }
    }

    /// Checks all bound values against their declared dependency slots.
    ///
    /// # Errors
    ///
    /// Returns the first shape, missing-value, or adapter contract error in
    /// signature order. Error metadata does not include raw dependency values.
    pub(crate) fn check_specs(&self, specs: &[DependencySpec]) -> Result<(), ExecutionError> {
        if self.values.len() != specs.len() || self.paths.is_some_and(|paths| paths.len() != specs.len()) {
            return Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation));
        }
        for (index, spec) in specs.iter().copied().enumerate() {
            let value = self.value(index)?;
            let error_kind = if value.is_missing() {
                if !spec.optional() {
                    ExecutionErrorKind::MissingRequiredDependencyValue
                } else {
                    continue;
                }
            } else if !spec.input().accepts(value) {
                ExecutionErrorKind::DependencyTypeMismatch
            } else {
                continue;
            };
            let path = self.dependency_path(index)?.clone();
            return Err(ExecutionError::new(error_kind)
                .with_dependency(spec.name())
                .with_path(path));
        }
        Ok(())
    }
}

/// Shared root path used when an adapter does not supply dependency paths.
static ROOT_PATH: ValidationPath = ValidationPath::root();

impl std::fmt::Debug for BoundValidationContext<'_> {
    /// Formats only the slot count so dependency values remain private.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BoundValidationContext")
            .field("slot_count", &self.values.len())
            .finish()
    }
}
