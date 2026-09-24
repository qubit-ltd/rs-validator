// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Reasons for skipping a validation occurrence.

/// Why a rule did not run.
///
/// # Examples
///
/// ```
/// use qubit_validator::SkipReason;
///
/// let reason = SkipReason::MissingOptional;
/// assert_eq!(reason, SkipReason::MissingOptional);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SkipReason {
    /// The target was explicitly absent and optional.
    MissingOptional,
    /// A required prerequisite had already failed.
    FailedPrerequisite,
}
