// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Named validator parameter values.

use crate::ValidationArgument;

/// One named validator parameter.
///
/// # Examples
///
/// ```
/// use qubit_validator::{NamedValidationArgument, ValidationArgument};
///
/// let argument = NamedValidationArgument::new(
///     "minimum",
///     ValidationArgument::Unsigned(3),
/// );
/// assert_eq!(argument.name(), "minimum");
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NamedValidationArgument<'a> {
    name: &'a str,
    value: ValidationArgument<'a>,
}

impl<'a> NamedValidationArgument<'a> {
    /// Creates a named parameter.
    ///
    /// # Panics
    ///
    /// Panics when `name` is empty.
    #[must_use]
    pub const fn new(name: &'a str, value: ValidationArgument<'a>) -> Self {
        assert!(!name.is_empty(), "validator parameter name cannot be empty");
        Self { name, value }
    }

    /// Returns the parameter name.
    #[must_use]
    pub const fn name(&self) -> &'a str {
        self.name
    }

    /// Returns the parameter value.
    #[must_use]
    pub const fn value(&self) -> ValidationArgument<'a> {
        self.value
    }
}
