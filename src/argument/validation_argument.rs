// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Domain-neutral validator parameter values.

/// One statically typed validator parameter value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
