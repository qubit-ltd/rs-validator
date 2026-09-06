// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Explicit outcomes returned by erased rule adapters.

use super::SkipReason;
use super::Violation;

/// Result of executing one prepared rule.
#[derive(Debug, PartialEq, Eq)]
pub enum RuleOutcome {
    /// The rule accepted the value.
    Valid,
    /// The rule rejected the value.
    Invalid(Vec<Violation>),
    /// The rule was intentionally skipped.
    Skipped {
        /// Why the rule was skipped.
        reason: SkipReason,
        /// Violations that caused a prerequisite skip.
        prerequisites: Vec<Violation>,
    },
}
