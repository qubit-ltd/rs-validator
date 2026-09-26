// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Domain-neutral validator parameter values.

/// One statically typed validator parameter value.
///
/// Borrowed strings and slices remain valid for the argument's lifetime.
/// Formatting redacts scalar contents and displays only collection lengths.
///
/// # Examples
///
/// ```
/// use qubit_validation_vocabulary::ValidationArgument;
///
/// let argument = ValidationArgument::String("admin");
/// assert_eq!(argument, ValidationArgument::String("admin"));
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ValidationArgument<'a> {
    /// A Boolean value.
    Bool(
        /// The supplied Boolean value.
        bool,
    ),
    /// A signed integer value.
    Integer(
        /// The supplied signed integer value.
        i128,
    ),
    /// An unsigned integer value.
    Unsigned(
        /// The supplied unsigned integer value.
        u128,
    ),
    /// A borrowed string value.
    String(
        /// The supplied string.
        &'a str,
    ),
    /// A borrowed Boolean list.
    BoolList(
        /// The supplied Boolean slice.
        &'a [bool],
    ),
    /// A borrowed signed integer list.
    IntegerList(
        /// The supplied signed integer slice.
        &'a [i128],
    ),
    /// A borrowed unsigned integer list.
    UnsignedList(
        /// The supplied unsigned integer slice.
        &'a [u128],
    ),
    /// A borrowed string list.
    StringList(
        /// The supplied slice of borrowed strings.
        &'a [&'a str],
    ),
}

impl std::fmt::Debug for ValidationArgument<'_> {
    /// Formats the value shape without exposing scalar contents.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(_) => formatter.write_str("Bool(<redacted>)"),
            Self::Integer(_) => formatter.write_str("Integer(<redacted>)"),
            Self::Unsigned(_) => formatter.write_str("Unsigned(<redacted>)"),
            Self::String(_) => formatter.write_str("String(<redacted>)"),
            Self::BoolList(values) => formatter.debug_struct("BoolList").field("len", &values.len()).finish(),
            Self::IntegerList(values) => formatter
                .debug_struct("IntegerList")
                .field("len", &values.len())
                .finish(),
            Self::UnsignedList(values) => formatter
                .debug_struct("UnsignedList")
                .field("len", &values.len())
                .finish(),
            Self::StringList(values) => formatter
                .debug_struct("StringList")
                .field("len", &values.len())
                .finish(),
        }
    }
}
