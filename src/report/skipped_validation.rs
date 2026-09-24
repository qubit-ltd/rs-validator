// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Explicitly skipped validation occurrences.

use super::SkipReason;
use super::ValidationOutcomeError;
use super::ValidationPath;
use super::Violation;

/// One validation occurrence that was skipped by the executor.
///
/// # Examples
///
/// ```
/// use qubit_validator::{SkipReason, SkippedValidation, ValidationPath};
///
/// let skipped = SkippedValidation::missing_optional(
///     0,
///     ValidationPath::root().with_field("optional"),
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
    /// Evidence violations for a failed prerequisite, empty for an absent
    /// optional target.
    prerequisites: Vec<Violation>,
}

impl SkippedValidation {
    /// Creates a record for an absent optional target.
    #[inline]
    pub fn missing_optional(occurrence: usize, path: ValidationPath) -> Self {
        Self {
            occurrence,
            path,
            reason: SkipReason::MissingOptional,
            prerequisites: Vec::new(),
        }
    }

    /// Creates a record for an occurrence skipped after prerequisite
    /// violations.
    ///
    /// # Errors
    ///
    /// Returns `EmptyPrerequisites` when no failed prerequisite is supplied.
    pub fn failed_prerequisite(
        occurrence: usize,
        path: ValidationPath,
        prerequisites: Vec<Violation>,
    ) -> Result<Self, ValidationOutcomeError> {
        if prerequisites.is_empty() {
            return Err(ValidationOutcomeError::EmptyPrerequisites);
        }
        Ok(Self {
            occurrence,
            path,
            reason: SkipReason::FailedPrerequisite,
            prerequisites,
        })
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

    /// Returns violations from prerequisites that prevented this occurrence.
    #[must_use = "inspect the prerequisite violations"]
    #[inline]
    pub fn prerequisites(&self) -> &[Violation] {
        &self.prerequisites
    }
}
