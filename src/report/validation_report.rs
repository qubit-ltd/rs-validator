// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Aggregated validation results.

use std::collections::HashSet;

use super::FailureId;
use super::RecordedOutcome;
use super::SkipReason;
use super::SkippedValidation;
use super::ValidationLimits;
use super::ValidationOutcomeError;
use super::Violation;
use super::next_report_id;

/// A bounded collection of validation violations and skipped occurrences.
///
/// Reports retain only structured violation metadata, never raw rejected input.
///
/// # Examples
///
/// ```
/// use qubit_validator::ValidationReport;
/// use qubit_validator::ValidatorId;
/// use qubit_validator::Violation;
/// use qubit_validator::ViolationCode;
/// use qubit_validator::ValidationOutcome;
/// use qubit_validator::ValidationOutcomeError;
/// use qubit_validator::ValidationPath;
///
/// let mut report = ValidationReport::new();
/// let violation = Violation::new(
///     ValidatorId::new("example.non_empty"),
///     ViolationCode::new("text.empty"),
/// );
/// let recorded = report.record_outcome(
///     0,
///     ValidationPath::root(),
///     ValidationOutcome::invalid(vec![violation])?,
/// )?;
/// assert!(recorded.complete());
/// assert!(!report.is_valid());
/// assert_eq!(report.violations().len(), 1);
/// # Ok::<(), ValidationOutcomeError>(())
/// ```
#[must_use]
pub struct ValidationReport {
    /// Accepted violations in occurrence order.
    violations: Vec<Violation>,
    /// Accepted skipped occurrences in occurrence order.
    skipped: Vec<SkippedValidation>,
    /// Unique identity used to reject failure references from other reports.
    report_id: u64,
    /// Whether configured limits prevented exhaustive collection.
    truncated: bool,
    /// Collection limits for all failures and skipped occurrences.
    limits: ValidationLimits,
}

impl ValidationReport {
    /// Creates an empty report with explicit collection limits.
    #[inline]
    pub fn with_limits(limits: ValidationLimits) -> Self {
        Self {
            violations: Vec::new(),
            skipped: Vec::new(),
            report_id: next_report_id(),
            truncated: false,
            limits,
        }
    }

    /// Creates an empty report.
    #[inline]
    pub fn new() -> Self {
        Self::with_limits(ValidationLimits {
            max_violations: None,
            max_skipped: None,
        })
    }

    /// Marks that a caller stopped validation before it was exhaustive.
    ///
    /// Use this when the execution policy stopped before the report's own
    /// collection limits rejected an outcome.
    pub fn mark_truncated(&mut self) {
        self.truncated = true;
    }

    /// Adds the result of one validation occurrence while preserving its
    /// invariants.
    ///
    /// Violations and skipped entries are stored in occurrence order and obey
    /// their respective report limits. The violation limit counts each
    /// retained invalid violation once; skipped outcomes refer to those
    /// failures by ID and consume no additional violation capacity. For an
    /// invalid outcome, `path` prefixes each relative violation path once.
    ///
    /// # Errors
    ///
    /// Returns an outcome shape error for malformed outcomes, duplicate
    /// prerequisite IDs, or IDs not retained by this report. Such an error
    /// leaves this report unchanged.
    ///
    /// # Returns
    ///
    /// Returns whether the whole outcome fits the configured limits and the
    /// IDs of failures retained from that outcome. A failed-prerequisite skip
    /// is retained only when every reference is valid and it fits the skip
    /// limit.
    pub fn record_outcome(
        &mut self,
        occurrence: usize,
        path: super::ValidationPath,
        outcome: crate::ValidationOutcome,
    ) -> Result<RecordedOutcome, ValidationOutcomeError> {
        match outcome {
            crate::ValidationOutcome::Valid => Ok(RecordedOutcome::new(true, Vec::new())),
            crate::ValidationOutcome::Invalid(violations) => {
                if violations.is_empty() {
                    return Err(ValidationOutcomeError::EmptyViolations);
                }
                let retained = violations.len().min(self.remaining_failure_capacity());
                let complete = retained == violations.len();
                let mut failure_ids = Vec::with_capacity(retained);
                for violation in violations.into_iter().take(retained) {
                    let full_path = path.concat(violation.path());
                    failure_ids.push(FailureId::new(self.report_id, self.violations.len()));
                    self.violations.push(violation.with_path(full_path));
                }
                if !complete {
                    self.mark_truncated();
                }
                Ok(RecordedOutcome::new(complete, failure_ids))
            }
            crate::ValidationOutcome::Skipped {
                reason: SkipReason::MissingOptional,
                prerequisites,
            } => {
                if !prerequisites.is_empty() {
                    return Err(ValidationOutcomeError::UnexpectedPrerequisites);
                }
                if !self.has_skipped_capacity() {
                    self.mark_truncated();
                    return Ok(RecordedOutcome::new(false, Vec::new()));
                }
                self.skipped.push(SkippedValidation::missing_optional(occurrence, path));
                Ok(RecordedOutcome::new(true, Vec::new()))
            }
            crate::ValidationOutcome::Skipped {
                reason: SkipReason::FailedPrerequisite,
                prerequisites,
            } => {
                if prerequisites.is_empty() {
                    return Err(ValidationOutcomeError::EmptyPrerequisites);
                }
                let mut unique = HashSet::with_capacity(prerequisites.len());
                for prerequisite in &prerequisites {
                    if !unique.insert(*prerequisite) {
                        return Err(ValidationOutcomeError::DuplicatePrerequisiteFailure);
                    }
                    if prerequisite.report != self.report_id || self.violations.get(prerequisite.index).is_none() {
                        return Err(ValidationOutcomeError::UnknownPrerequisiteFailure);
                    }
                }
                if !self.has_skipped_capacity() {
                    self.mark_truncated();
                    return Ok(RecordedOutcome::new(false, Vec::new()));
                }
                let skipped = SkippedValidation::failed_prerequisite(occurrence, path, prerequisites)?;
                self.skipped.push(skipped);
                Ok(RecordedOutcome::new(true, Vec::new()))
            }
        }
    }

    /// Returns the remaining shared capacity for top-level and prerequisite
    /// violations, or the largest possible count when unbounded.
    fn remaining_failure_capacity(&self) -> usize {
        self.limits
            .max_violations
            .map_or(usize::MAX, |limit| limit.saturating_sub(self.violations.len()))
    }

    /// Returns whether one more skipped occurrence fits the configured limit.
    fn has_skipped_capacity(&self) -> bool {
        self.limits.max_skipped.is_none_or(|limit| self.skipped.len() < limit)
    }

    /// Returns whether validation completed with no failures.
    ///
    /// # Returns
    ///
    /// Returns `true` only when there are no violations, validation was not
    /// truncated, and no prerequisite failure caused an occurrence to be
    /// skipped.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty()
            && !self.truncated
            && self
                .skipped
                .iter()
                .all(|item| item.reason() != SkipReason::FailedPrerequisite)
    }

    /// Returns all violations in occurrence order.
    #[must_use = "the collected violations should be inspected"]
    #[inline]
    pub fn violations(&self) -> &[Violation] {
        &self.violations
    }

    /// Iterates over retained original violations in occurrence order.
    ///
    /// # Returns
    ///
    /// Every retained original failure in the same count reported by
    /// [`Self::failure_count`].
    #[must_use = "inspect all retained failure evidence"]
    pub fn failures(&self) -> impl Iterator<Item = &Violation> + '_ {
        self.violations.iter()
    }

    /// Returns the number of retained original violations.
    #[must_use]
    #[inline]
    pub const fn failure_count(&self) -> usize {
        self.violations.len()
    }

    /// Returns the violation associated with a failure ID from this report.
    ///
    /// # Returns
    ///
    /// Returns `Some` when `id` was issued by this report and remains
    /// retained; otherwise returns `None`.
    #[must_use]
    pub fn failure(&self, id: FailureId) -> Option<&Violation> {
        if id.report == self.report_id {
            self.violations.get(id.index)
        } else {
            None
        }
    }

    /// Returns all skipped occurrences in occurrence order.
    #[must_use = "the skipped occurrences should be inspected"]
    #[inline]
    pub fn skipped(&self) -> &[SkippedValidation] {
        &self.skipped
    }

    /// Returns whether validation was truncated.
    ///
    /// # Returns
    ///
    /// Returns `true` when a collection limit prevented exhaustive results.
    #[must_use]
    #[inline]
    pub const fn is_truncated(&self) -> bool {
        self.truncated
    }

    /// Returns the configured collection limits.
    #[must_use]
    #[inline]
    pub const fn limits(&self) -> ValidationLimits {
        self.limits
    }
}

impl Default for ValidationReport {
    /// Creates an unbounded empty report.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ValidationReport {
    /// Formats counts and truncation state without exposing report contents.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ValidationReport")
            .field("violation_count", &self.failure_count())
            .field("skipped_count", &self.skipped.len())
            .field("truncated", &self.truncated)
            .finish()
    }
}

impl std::fmt::Display for ValidationReport {
    /// Formats counts and truncation state without exposing report contents.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "validation report: {} violation(s), {} skipped, truncated={}",
            self.failure_count(),
            self.skipped.len(),
            self.truncated,
        )
    }
}
