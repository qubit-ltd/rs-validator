// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Outcomes returned by prepared validators.

use super::ViolationDraft;
use crate::SkipReason;
use crate::Violation;
/// Outcome produced by a prepared validator.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PreparedOutcome {
    /// Input is valid.
    Valid,
    /// Input is invalid with draft violations.
    Invalid(Vec<ViolationDraft>),
    /// Execution was skipped.
    Skipped {
        /// Reason for skipping.
        reason: SkipReason,
        /// Prerequisite violations.
        prerequisites: Vec<Violation>,
    },
}
