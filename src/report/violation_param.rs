// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Restricted vocabulary for structured validation diagnostic parameters.

/// A restricted vocabulary for diagnostic parameters. The type limits how a
/// value is represented; it cannot prove where the value came from. Callers
/// must supply only program-declared tokens or rule configuration values, never
/// rejected input or sensitive values derived from it.
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
    /// A token with a static lifetime; the lifetime does not prove its source.
    Token(
        /// Callers must provide a program-declared token, not rejected input.
        &'static str,
    ),
}
