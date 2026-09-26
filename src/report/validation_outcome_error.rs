// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Construction and recording errors for validation outcomes.

/// An outcome violates the invariant required by its declared variant.
///
/// # Examples
///
/// ```
/// use qubit_validator::{ValidationOutcome, ValidationOutcomeError};
///
/// assert_eq!(
///     ValidationOutcome::invalid(Vec::new()),
///     Err(ValidationOutcomeError::EmptyViolations),
/// );
/// ```
#[must_use]
#[derive(Clone, Copy, Debug, Eq, thiserror::Error, PartialEq)]
#[non_exhaustive]
pub enum ValidationOutcomeError {
    /// An outcome was submitted before the last successfully recorded
    /// occurrence.
    #[error("validation outcomes must be recorded in non-decreasing occurrence order")]
    OutOfOrderOccurrence,
    /// An invalid outcome contains no violations.
    #[error("an invalid outcome must contain at least one violation")]
    EmptyViolations,
    /// A failed-prerequisite outcome contains no prerequisite failure IDs.
    #[error("a failed-prerequisite outcome must contain at least one prerequisite failure ID")]
    EmptyPrerequisites,
    /// A missing-optional outcome contains prerequisite violations.
    #[error("a missing-optional outcome cannot contain prerequisite violations")]
    UnexpectedPrerequisites,
    /// A failed-prerequisite outcome contains an ID not retained by this
    /// report.
    #[error("a prerequisite failure ID is unknown to this report")]
    UnknownPrerequisiteFailure,
    /// A failed-prerequisite outcome references one failure more than once.
    #[error("a prerequisite failure ID is repeated")]
    DuplicatePrerequisiteFailure,
}
