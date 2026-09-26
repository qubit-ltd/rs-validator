// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Receipt returned after recording one validation outcome.

use super::FailureId;

/// Describes whether an outcome fit the report limits and identifies its
/// retained failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedOutcome {
    complete: bool,
    failure_ids: Vec<FailureId>,
}

impl RecordedOutcome {
    /// Creates a receipt for report recording.
    pub(crate) fn new(complete: bool, failure_ids: Vec<FailureId>) -> Self {
        Self { complete, failure_ids }
    }

    /// Returns whether every part of the outcome fit the configured limits.
    #[must_use]
    #[inline]
    pub const fn complete(&self) -> bool {
        self.complete
    }

    /// Returns IDs for failures retained from this outcome.
    ///
    /// Skipped and valid outcomes return an empty slice.
    #[must_use]
    #[inline]
    pub fn failure_ids(&self) -> &[FailureId] {
        &self.failure_ids
    }
}
