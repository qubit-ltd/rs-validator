// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Named validator parameters.

use crate::ValidationArgument;

/// One named validator parameter borrowing its name and value.
///
/// # Type Parameters
///
/// - `'a`: Lifetime of the borrowed name and any string or slice in the value.
///
/// # Examples
///
/// ```
/// use qubit_validation_vocabulary::{NamedValidationArgument, ValidationArgument};
///
/// let argument = NamedValidationArgument::new("minimum", ValidationArgument::Unsigned(3));
/// assert_eq!(argument.name(), "minimum");
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct NamedValidationArgument<'a> {
    /// Name used by the validator's parameter schema.
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
    /// A borrowed parameter retaining the supplied name and value.
    ///
    /// # Panics
    ///
    /// Panics when `name` is empty.
    #[must_use]
    pub const fn new(name: &'a str, value: ValidationArgument<'a>) -> Self {
        assert!(!name.is_empty(), "validator parameter name cannot be empty");
        Self { name, value }
    }

    /// Returns the borrowed parameter name.
    ///
    /// # Returns
    ///
    /// The non-empty name supplied to [`Self::new`].
    #[must_use]
    pub const fn name(&self) -> &'a str {
        self.name
    }

    /// Returns the copyable typed parameter value.
    ///
    /// # Returns
    ///
    /// The supplied value, with any borrowed data still tied to `'a`.
    #[must_use]
    pub const fn value(&self) -> ValidationArgument<'a> {
        self.value
    }
}

impl std::fmt::Debug for NamedValidationArgument<'_> {
    /// Formats the value without exposing the parameter name or contents.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NamedValidationArgument")
            .field("value", &self.value)
            .finish()
    }
}
