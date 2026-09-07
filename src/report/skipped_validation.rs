// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Explicitly skipped validation occurrences.

use super::SkipReason;
use super::ValidationPath;

/// One validation occurrence that was skipped by the executor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkippedValidation {
    occurrence: usize,
    path: ValidationPath,
    reason: SkipReason,
}

impl SkippedValidation {
    /// Creates a skipped validation record.
    #[must_use]
    pub const fn new(
        occurrence: usize,
        path: ValidationPath,
        reason: SkipReason,
    ) -> Self {
        Self {
            occurrence,
            path,
            reason,
        }
    }

    /// Returns the declaration occurrence.
    #[must_use]
    pub const fn occurrence(&self) -> usize {
        self.occurrence
    }

    /// Returns the skipped rule path.
    #[must_use]
    pub const fn path(&self) -> &ValidationPath {
        &self.path
    }

    /// Returns the skip reason.
    #[must_use]
    pub const fn reason(&self) -> SkipReason {
        self.reason
    }
}
