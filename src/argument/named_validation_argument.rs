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
///
/// # Type Parameters
///
/// - `'a`: Lifetime of the borrowed parameter name and value.
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
    /// Name used by a validator's parameter schema.
    name: &'a str,
    /// Typed value supplied for the named parameter.
    value: ValidationArgument<'a>,
}

impl<'a> NamedValidationArgument<'a> {
    /// Creates a named parameter.
    ///
    /// # Parameters
    ///
    /// - `name`: Non-empty name declared by the validator's parameter schema.
    /// - `value`: Typed value supplied for that parameter.
    ///
    /// # Returns
    ///
    /// A named parameter retaining the supplied borrowed data.
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
    ///
    /// # Returns
    ///
    /// The name borrowed from this argument.
    #[must_use]
    #[inline]
    pub const fn name(&self) -> &'a str {
        self.name
    }

    /// Returns the parameter value.
    ///
    /// # Returns
    ///
    /// The copyable typed value, which may contain borrowed data.
    #[must_use]
    #[inline]
    pub const fn value(&self) -> ValidationArgument<'a> {
        self.value
    }
}
