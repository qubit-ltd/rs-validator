// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Ordered, shape-checked dependency values for one validation call.

use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use super::DependencySpec;
use super::ExecutionError;
use super::ExecutionErrorKind;
use super::ValidationPath;
use super::ValidationValue;

/// Borrowed dependency slots used during a synchronous validation call.
///
/// Optional paths are stored in slot order exactly as supplied by the caller.
/// This type does not infer a coordinate system or add the current target's
/// path. When paths are omitted, [`Self::dependency_path`] returns the root
/// path as a default location; that does not identify the target or report
/// root.
///
/// # Examples
///
/// ```
/// use qubit_validator::{BoundValidationContext, ValidationValue};
///
/// let owner = String::from("member@example.test");
/// let values = [ValidationValue::Text(&owner)];
/// let context = BoundValidationContext::new(&values);
/// assert_eq!(context.text(0)?, "member@example.test");
/// # Ok::<(), qubit_validator::ExecutionError>(())
/// ```
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

    /// Creates a context with caller-supplied dependency paths.
    ///
    /// Paths are associated with values by slot index and kept unchanged. The
    /// caller chooses their coordinate system; this method does not add a
    /// current-target prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// use qubit_validator::{BoundValidationContext, ValidationPath, ValidationValue};
    ///
    /// let values = [ValidationValue::Text("ada@example.test")];
    /// let paths = [ValidationPath::root().with_field("profile").with_field("email")];
    /// let context = BoundValidationContext::new_with_paths(&values, &paths)?;
    /// assert_eq!(context.dependency_path(0)?.render(), "profile.email");
    /// # Ok::<(), qubit_validator::ExecutionError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an adapter contract error when the two slices differ in length.
    ///
    /// # Returns
    ///
    /// Returns a context whose values and paths occupy matching slots.
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
    ///
    /// # Returns
    ///
    /// Returns a copy of the value stored at `index`.
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
    ///
    /// # Returns
    ///
    /// Returns the stored path unchanged, or the root path when no paths were
    /// supplied. The default root is only a fallback location and does not
    /// establish that the dependency is the current target.
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
    /// # Type Parameters
    ///
    /// - `T`: Concrete dependency type to borrow; it must be `'static` for
    ///   `Any` downcasting.
    ///
    /// # Errors
    ///
    /// Returns a shape, missing-value, or slot error.
    ///
    /// # Returns
    ///
    /// Returns a borrow of the value when it has concrete type `T`.
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
    /// # Type Parameters
    ///
    /// - `T`: Concrete dependency type to borrow; it must be `'static` for
    ///   `Any` downcasting.
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
    ///
    /// # Returns
    ///
    /// Returns the borrowed text dependency.
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
    /// signature order. Shape errors retain the supplied slot path, or its
    /// default root path. Error metadata does not include raw dependency
    /// values.
    ///
    /// # Returns
    ///
    /// Returns `()` when every slot satisfies its declared dependency.
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

impl Debug for BoundValidationContext<'_> {
    /// Formats only the slot count so dependency values remain private.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter
            .debug_struct("BoundValidationContext")
            .field("slot_count", &self.values.len())
            .finish()
    }
}
