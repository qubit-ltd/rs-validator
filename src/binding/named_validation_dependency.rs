// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Named borrowed dependency values for direct bound-validator calls.

use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result;

use crate::ValidationPath;
use crate::ValidationValue;

/// One dependency value identified by its signature name.
///
/// Names come from the validator declaration. Values and paths remain borrowed
/// for the validation call and are never retained by the binding. An optional
/// path is caller-selected and remains attached to its named slot when inputs
/// are reordered; it is not prefixed with the current target path.
///
/// # Type Parameters
///
/// - `'a`: Lifetime shared by the borrowed value and optional path.
///
/// # Parameters
/// - `name`: Static dependency name declared by the validator signature.
/// - `value`: Borrowed value for the dependency slot.
///
/// # Examples
///
/// ```
/// use qubit_validator::{NamedValidationDependency, ValidationValue};
///
/// let dependency = NamedValidationDependency::new("email", ValidationValue::Text("a@example.test"));
/// assert!(!format!("{dependency:?}").contains("a@example.test"));
/// ```
#[derive(Clone, Copy)]
pub struct NamedValidationDependency<'a> {
    /// Static name declared by the selected validator signature.
    pub(super) name: &'static str,
    /// Borrowed value supplied for the dependency slot.
    pub(super) value: ValidationValue<'a>,
    /// Optional location used in structured dependency errors.
    pub(super) path: Option<&'a ValidationPath>,
}

impl<'a> NamedValidationDependency<'a> {
    /// Creates a named dependency value without a specific path.
    ///
    /// # Parameters
    /// - `name`: Static name from the validator signature.
    /// - `value`: Borrowed value for the dependency.
    ///
    /// # Returns
    /// A named borrowed dependency with no associated path.
    #[must_use]
    #[inline]
    pub const fn new(name: &'static str, value: ValidationValue<'a>) -> Self {
        Self {
            name,
            value,
            path: None,
        }
    }

    /// Associates a caller-supplied path with this dependency value.
    ///
    /// The path is kept unchanged and follows this named slot if
    /// [`crate::BoundValidator::validate_named`] reorders the inputs. This
    /// method does not add a current-target prefix.
    ///
    /// # Parameters
    /// - `path`: Borrowed path identifying the dependency in the coordinate
    ///   system selected by the caller.
    ///
    /// # Returns
    /// The dependency associated with `path`.
    #[must_use]
    #[inline]
    pub const fn with_path(mut self, path: &'a ValidationPath) -> Self {
        self.path = Some(path);
        self
    }
}

impl Debug for NamedValidationDependency<'_> {
    /// Formats only dependency shape without exposing values or paths.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("NamedValidationDependency")
            .field("has_path", &self.path.is_some())
            .finish_non_exhaustive()
    }
}
