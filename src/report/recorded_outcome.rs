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
///
/// # Examples
///
/// ```
/// use qubit_validator::{RecordedOutcome, ValidationOutcome, ValidationPath, ValidationReport};
///
/// let mut report = ValidationReport::new();
/// let receipt: RecordedOutcome = report.record_outcome(
///     0,
///     ValidationPath::root(),
///     ValidationOutcome::valid(),
/// )?;
/// assert!(receipt.complete());
/// assert!(receipt.failure_ids().is_empty());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
    ///
    /// This reports collection completeness, not validation success. It is
    /// `false` when a violation or skipped occurrence could not be retained.
    #[must_use]
    #[inline]
    pub const fn complete(&self) -> bool {
        self.complete
    }

    /// Returns IDs for failures retained from this outcome.
    ///
    /// Returns only failures newly retained from this outcome. Valid and
    /// skipped outcomes return an empty slice; an invalid outcome can also
    /// return an empty slice when no failure from it fit the report limit.
    #[must_use]
    #[inline]
    pub fn failure_ids(&self) -> &[FailureId] {
        &self.failure_ids
    }
}
