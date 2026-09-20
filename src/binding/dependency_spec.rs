//! Dependency slot declarations for prepared validators.

use super::InputType;

/// Describes one ordered dependency slot supplied to a validator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DependencySpec {
    name: &'static str,
    input: InputType,
    optional: bool,
}

impl DependencySpec {
    /// Creates a dependency declaration.
    #[must_use]
    pub const fn new(name: &'static str, input: InputType, optional: bool) -> Self {
        assert!(!name.is_empty(), "dependency name cannot be empty");
        Self { name, input, optional }
    }

    /// Returns the logical slot name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Returns the required input shape.
    #[must_use]
    pub const fn input(self) -> InputType {
        self.input
    }

    /// Returns whether an explicit missing value is accepted.
    #[must_use]
    pub const fn optional(self) -> bool {
        self.optional
    }
}
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
