// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Safe, structured validation violation parameters.

/// A parameter that is safe to carry in a public violation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViolationParam {
    /// A boolean parameter.
    Bool(bool),
    /// A signed integer parameter.
    Signed(i128),
    /// An unsigned integer parameter.
    Unsigned(u128),
    /// A static, program-declared token.
    Token(&'static str),
}
