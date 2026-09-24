// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Violation code protocol errors.

use thiserror::Error;

/// The reason a candidate string violates the stable violation-code protocol.
///
/// # Examples
///
/// ```
/// use qubit_validator::{ViolationCode, ViolationCodeError};
///
/// let error = ViolationCode::try_new("text..blank").unwrap_err();
/// assert_eq!(error, ViolationCodeError::EmptySegment);
/// ```
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
pub enum ViolationCodeError {
    /// The complete code is empty.
    #[error("violation code cannot be empty")]
    Empty,
    /// One dot-separated code segment is empty.
    #[error("violation code contains an empty segment")]
    EmptySegment,
    /// One code segment contains an invalid character or initial byte.
    #[error("violation code contains an invalid segment")]
    InvalidSegment,
}
