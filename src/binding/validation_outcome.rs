// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Public outcomes produced by bound validators.

use crate::SkipReason;
use crate::ValidationOutcomeError;
use crate::Violation;
/// The observable result of executing one bound validator occurrence.
///
/// # Examples
///
/// ```
/// use qubit_validator::{ValidationOutcome, ValidatorId, Violation, ViolationCode};
///
/// let accepted = ValidationOutcome::valid();
/// let rejected = ValidationOutcome::invalid(vec![Violation::new(
///     ValidatorId::new("text.required"),
///     ViolationCode::new("text.blank"),
/// )])?;
/// assert!(matches!(accepted, ValidationOutcome::Valid));
/// assert!(matches!(rejected, ValidationOutcome::Invalid(violations) if violations.len() == 1));
/// # Ok::<(), qubit_validator::ValidationOutcomeError>(())
/// ```
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

impl ValidationOutcome {
    /// Creates a successful validation outcome.
    #[inline]
    pub const fn valid() -> Self {
        Self::Valid
    }

    /// Creates an invalid outcome with one or more violations.
    ///
    /// # Errors
    ///
    /// Returns `EmptyViolations` when `violations` is empty.
    pub fn invalid(violations: Vec<Violation>) -> Result<Self, ValidationOutcomeError> {
        if violations.is_empty() {
            return Err(ValidationOutcomeError::EmptyViolations);
        }
        Ok(Self::Invalid(violations))
    }

    /// Creates an outcome skipped because the optional target is absent.
    #[inline]
    pub fn missing_optional() -> Self {
        Self::Skipped {
            reason: SkipReason::MissingOptional,
            prerequisites: Vec::new(),
        }
    }

    /// Creates an outcome skipped after at least one prerequisite failed.
    ///
    /// # Errors
    ///
    /// Returns `EmptyPrerequisites` when `prerequisites` is empty.
    pub fn failed_prerequisite(prerequisites: Vec<Violation>) -> Result<Self, ValidationOutcomeError> {
        if prerequisites.is_empty() {
            return Err(ValidationOutcomeError::EmptyPrerequisites);
        }
        Ok(Self::Skipped {
            reason: SkipReason::FailedPrerequisite,
            prerequisites,
        })
    }
}
