// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Validator ID errors.

use thiserror::Error;

/// A stable validator ID protocol violation.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
pub enum ValidatorIdError {
    /// The complete ID is empty.
    #[error("validator ID cannot be empty")]
    Empty,
    /// One dot-separated segment is empty.
    #[error("validator ID contains an empty segment")]
    EmptySegment,
    /// One segment contains an invalid character or initial byte.
    #[error("validator ID contains an invalid segment")]
    InvalidSegment,
}
