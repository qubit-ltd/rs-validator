// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Type-erased results produced by prepared validators.

use super::ViolationDraft;
use crate::SkipReason;
use crate::Violation;
/// The type-erased result produced before rule identity is attached.
#[must_use]
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PreparedOutcome {
    /// Input is valid.
    Valid,
    /// Input is invalid with draft violations.
    Invalid(
        /// One or more safe draft violations produced by the validator.
        Vec<ViolationDraft>,
    ),
    /// Execution was skipped.
    Skipped {
        /// Reason for skipping.
        reason: SkipReason,
        /// Prerequisite violations.
        prerequisites: Vec<Violation>,
    },
}
