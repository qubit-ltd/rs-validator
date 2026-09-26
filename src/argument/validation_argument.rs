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
/// # Type Parameters
///
/// - `'a`: Lifetime of any borrowed string or slice carried by the value.
///
/// # Examples
///
/// ```
/// use qubit_validator::ValidationArgument;
///
/// let argument = ValidationArgument::String("admin");
/// assert_eq!(argument, ValidationArgument::String("admin"));
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
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
    /// A string value.
    String(
        /// The supplied borrowed string.
        &'a str,
    ),
    /// A Boolean list.
    BoolList(
        /// The supplied borrowed Boolean slice.
        &'a [bool],
    ),
    /// A signed integer list.
    IntegerList(
        /// The supplied borrowed signed integer slice.
        &'a [i128],
    ),
    /// An unsigned integer list.
    UnsignedList(
        /// The supplied borrowed unsigned integer slice.
        &'a [u128],
    ),
    /// A string list.
    StringList(
        /// The supplied borrowed slice of borrowed strings.
        &'a [&'a str],
    ),
}

impl std::fmt::Debug for ValidationArgument<'_> {
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
