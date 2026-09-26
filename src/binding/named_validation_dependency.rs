// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Named borrowed dependency values for direct bound-validator calls.

use crate::ValidationPath;
use crate::ValidationValue;

/// One dependency value identified by its signature name.
///
/// Names come from the validator declaration. Values and paths remain borrowed
/// for the validation call and are never retained by the binding.
///
/// # Parameters
/// - `name`: Static dependency name declared by the validator signature.
/// - `value`: Borrowed value for the dependency slot.
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

    /// Associates a model path with this dependency value.
    ///
    /// # Parameters
    /// - `path`: Borrowed path identifying the dependency location.
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

impl std::fmt::Debug for NamedValidationDependency<'_> {
    /// Formats only dependency shape without exposing values or paths.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NamedValidationDependency")
            .field("has_path", &self.path.is_some())
            .finish_non_exhaustive()
    }
}
