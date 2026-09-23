// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Errors produced by the shared dotted identifier parser.

/// A structural reason a dotted identifier does not match the protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DottedIdentifierError {
    /// The complete identifier is empty.
    Empty,
    /// A dot-separated identifier segment is empty.
    EmptySegment,
    /// A segment contains an invalid character or initial byte.
    InvalidSegment,
}
