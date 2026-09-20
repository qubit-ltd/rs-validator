// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Named validator parameter values.

use crate::ValidationArgument;

/// One named validator parameter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NamedValidationArgument<'a> {
    /// Name used by a validator's parameter schema.
    name: &'a str,
    /// Typed value supplied for the named parameter.
    value: ValidationArgument<'a>,
}

impl<'a> NamedValidationArgument<'a> {
    /// Creates a named parameter.
    ///
    /// # Panics
    ///
    /// Panics when `name` is empty.
    #[must_use]
    #[inline]
    pub const fn new(name: &'a str, value: ValidationArgument<'a>) -> Self {
        assert!(!name.is_empty(), "validator parameter name cannot be empty");
        Self { name, value }
    }

    /// Returns the parameter name.
    #[must_use]
    #[inline]
    pub const fn name(&self) -> &'a str {
        self.name
    }

    /// Returns the parameter value.
    #[must_use]
    #[inline]
    pub const fn value(&self) -> ValidationArgument<'a> {
        self.value
    }
}
