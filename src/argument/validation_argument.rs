// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Domain-neutral validator parameter values.

/// One statically typed validator parameter value.
///
/// # Examples
///
/// ```
/// use qubit_validator::ValidationArgument;
///
/// let argument = ValidationArgument::String("admin");
/// assert_eq!(argument, ValidationArgument::String("admin"));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationArgument<'a> {
    /// A Boolean value.
    Bool(bool),
    /// A signed integer value.
    Integer(i128),
    /// An unsigned integer value.
    Unsigned(u128),
    /// A string value.
    String(&'a str),
    /// A Boolean list.
    BoolList(&'a [bool]),
    /// A signed integer list.
    IntegerList(&'a [i128]),
    /// An unsigned integer list.
    UnsignedList(&'a [u128]),
    /// A string list.
    StringList(&'a [&'a str]),
}
