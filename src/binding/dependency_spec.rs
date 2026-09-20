// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Dependency slot declarations for prepared validators.

use super::InputType;

/// Describes one ordered dependency slot supplied to a validator.
///
/// # Examples
///
/// ```
/// use qubit_validator::{DependencySpec, InputType};
///
/// let dependency = DependencySpec::new("credential", InputType::Text, false);
/// assert_eq!(dependency.name(), "credential");
/// assert!(!dependency.optional());
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DependencySpec {
    /// Logical slot name used in diagnostics.
    name: &'static str,
    /// Erased value shape required by the slot.
    input: InputType,
    /// Whether the explicit missing marker is accepted.
    optional: bool,
}

impl DependencySpec {
    /// Creates a dependency declaration.
    ///
    /// # Panics
    ///
    /// Panics when `name` is empty.
    #[must_use]
    #[inline]
    pub const fn new(name: &'static str, input: InputType, optional: bool) -> Self {
        assert!(!name.is_empty(), "dependency name cannot be empty");
        Self { name, input, optional }
    }

    /// Returns the logical slot name.
    #[must_use]
    #[inline]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Returns the required input shape.
    #[must_use]
    #[inline]
    pub const fn input(self) -> InputType {
        self.input
    }

    /// Returns whether an explicit missing value is accepted.
    ///
    /// # Returns
    ///
    /// Returns `true` when the dependency accepts
    /// [`super::ValidationValue::Missing`].
    #[must_use]
    #[inline]
    pub const fn optional(self) -> bool {
        self.optional
    }
}
