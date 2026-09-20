// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Explicitly skipped validation occurrences.

use super::SkipReason;
use super::ValidationPath;

/// One validation occurrence that was skipped by the executor.
///
/// # Examples
///
/// ```
/// use qubit_validator::{SkipReason, SkippedValidation, ValidationPath};
///
/// let skipped = SkippedValidation::new(
///     0,
///     ValidationPath::root().with_field("optional"),
///     SkipReason::MissingOptional,
/// );
/// assert_eq!(skipped.reason(), SkipReason::MissingOptional);
/// ```
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkippedValidation {
    /// Zero-based declaration occurrence assigned by the executor.
    occurrence: usize,
    /// Structured path of the skipped target.
    path: ValidationPath,
    /// Policy reason the occurrence did not execute.
    reason: SkipReason,
}

impl SkippedValidation {
    /// Creates a skipped validation record.
    #[inline]
    pub const fn new(occurrence: usize, path: ValidationPath, reason: SkipReason) -> Self {
        Self {
            occurrence,
            path,
            reason,
        }
    }

    /// Returns the declaration occurrence.
    #[must_use]
    #[inline]
    pub const fn occurrence(&self) -> usize {
        self.occurrence
    }

    /// Returns the skipped rule path.
    #[must_use]
    #[inline]
    pub const fn path(&self) -> &ValidationPath {
        &self.path
    }

    /// Returns the skip reason.
    #[must_use]
    #[inline]
    pub const fn reason(&self) -> SkipReason {
        self.reason
    }
}
