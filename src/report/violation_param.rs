// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Safe, structured validation violation parameters.

/// A parameter that is safe to carry in a public violation.
///
/// # Examples
///
/// ```
/// use qubit_validator::ViolationParam;
///
/// let parameter = ViolationParam::Unsigned(3);
/// assert_eq!(parameter, ViolationParam::Unsigned(3));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ViolationParam {
    /// A boolean parameter.
    Bool(
        /// Safe Boolean metadata.
        bool,
    ),
    /// A signed integer parameter.
    Signed(
        /// Safe signed integer metadata.
        i128,
    ),
    /// An unsigned integer parameter.
    Unsigned(
        /// Safe unsigned integer metadata.
        u128,
    ),
    /// A static, program-declared token.
    Token(
        /// Static, program-declared token rather than raw input.
        &'static str,
    ),
}
