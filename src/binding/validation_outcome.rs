// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Public outcomes produced by bound validators.

use crate::SkipReason;
use crate::Violation;
/// The observable result of executing one bound validator occurrence.
#[must_use]
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ValidationOutcome {
    /// Input is valid.
    Valid,
    /// Input is invalid.
    Invalid(
        /// One or more safe violations carrying no raw rejected value.
        Vec<Violation>,
    ),
    /// Execution was skipped.
    Skipped {
        /// Reason for skipping.
        reason: SkipReason,
        /// Prerequisite violations.
        prerequisites: Vec<Violation>,
    },
}
