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
use crate::ValidationOutcome;
use crate::ValidationOutcomeError;
use crate::ValidatorId;
use crate::Violation;
/// The type-erased result produced before rule identity is attached.
///
/// # Examples
///
/// ```
/// use qubit_validator::{PreparedOutcome, ViolationCode, ViolationDraft};
///
/// let accepted = PreparedOutcome::valid();
/// let rejected = PreparedOutcome::invalid(vec![
///     ViolationDraft::new(ViolationCode::new("text.blank")),
/// ])?;
/// assert!(matches!(accepted, PreparedOutcome::Valid));
/// assert!(matches!(rejected, PreparedOutcome::Invalid(drafts) if drafts.len() == 1));
/// # Ok::<(), qubit_validator::ValidationOutcomeError>(())
/// ```
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

impl PreparedOutcome {
    /// Creates a successful prepared outcome.
    #[inline]
    pub const fn valid() -> Self {
        Self::Valid
    }

    /// Creates an invalid prepared outcome with one or more violation drafts.
    ///
    /// # Errors
    ///
    /// Returns `EmptyViolations` when `violations` is empty.
    pub fn invalid(violations: Vec<ViolationDraft>) -> Result<Self, ValidationOutcomeError> {
        if violations.is_empty() {
            return Err(ValidationOutcomeError::EmptyViolations);
        }
        Ok(Self::Invalid(violations))
    }

    /// Creates a prepared outcome skipped because the optional target is
    /// absent.
    #[inline]
    pub fn missing_optional() -> Self {
        Self::Skipped {
            reason: SkipReason::MissingOptional,
            prerequisites: Vec::new(),
        }
    }

    /// Creates a prepared outcome skipped after a prerequisite failed.
    ///
    /// # Errors
    ///
    /// Returns `EmptyPrerequisites` when `prerequisites` is empty.
    pub fn failed_prerequisite(
        prerequisites: Vec<Violation>,
    ) -> Result<Self, ValidationOutcomeError> {
        if prerequisites.is_empty() {
            return Err(ValidationOutcomeError::EmptyPrerequisites);
        }
        Ok(Self::Skipped {
            reason: SkipReason::FailedPrerequisite,
            prerequisites,
        })
    }

    /// Attaches a bound rule identity to draft violations.
    ///
    /// # Errors
    ///
    /// Returns an outcome contract error when an invalid result is empty or
    /// a skipped result has prerequisites inconsistent with its reason.
    pub fn into_bound(
        self,
        rule_id: ValidatorId,
    ) -> Result<ValidationOutcome, ValidationOutcomeError> {
        match self {
            Self::Valid => Ok(ValidationOutcome::Valid),
            Self::Invalid(drafts) => ValidationOutcome::invalid(
                drafts
                    .into_iter()
                    .map(|draft| Violation::from_draft(rule_id, draft))
                    .collect(),
            ),
            Self::Skipped {
                reason,
                prerequisites,
            } => match reason {
                SkipReason::MissingOptional if !prerequisites.is_empty() => {
                    Err(ValidationOutcomeError::UnexpectedPrerequisites)
                }
                SkipReason::FailedPrerequisite if prerequisites.is_empty() => {
                    Err(ValidationOutcomeError::EmptyPrerequisites)
                }
                _ => Ok(ValidationOutcome::Skipped {
                    reason,
                    prerequisites,
                }),
            },
        }
    }
}
