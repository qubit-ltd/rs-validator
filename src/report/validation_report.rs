// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Aggregated validation results.

use std::collections::HashSet;
use std::fmt;

use super::FailureId;
use super::RecordedOutcome;
use super::SkipReason;
use super::SkippedValidation;
use super::ValidationLimits;
use super::ValidationOutcomeError;
use super::ValidationPath;
use super::Violation;
use super::next_report_id;
use crate::ValidationOutcome;

/// A bounded collection of validation violations and skipped occurrences.
///
/// Reports retain structured violation metadata. Callers must ensure that
/// violation parameters contain no rejected input or sensitive values derived
/// from it.
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
    /// Occurrence for each accepted violation, indexed like `violations`.
    failure_occurrences: Vec<usize>,
    /// Accepted skipped occurrences in occurrence order.
    skipped: Vec<SkippedValidation>,
    /// Unique identity used to reject failure references from other reports.
    report_id: u64,
    /// Whether configured limits prevented exhaustive collection.
    truncated: bool,
    /// Greatest occurrence successfully submitted to this report.
    last_recorded_occurrence: Option<usize>,
    /// Collection limits for all failures and skipped occurrences.
    limits: ValidationLimits,
}

impl ValidationReport {
    /// Creates an empty report with explicit collection limits.
    #[inline]
    pub fn with_limits(limits: ValidationLimits) -> Self {
        Self {
            violations: Vec::new(),
            failure_occurrences: Vec::new(),
            skipped: Vec::new(),
            report_id: next_report_id(),
            truncated: false,
            last_recorded_occurrence: None,
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
    /// Occurrences must be submitted in non-decreasing order; multiple outcomes
    /// may use the same occurrence. Violations and skipped entries are stored
    /// in submission order and obey their respective report limits. The
    /// violation limit counts each retained invalid violation once; skipped
    /// outcomes refer to those failures by ID and consume no additional
    /// violation capacity. For an invalid outcome, `path` prefixes each
    /// relative violation path once.
    ///
    /// # Parameters
    ///
    /// - `occurrence`: Non-decreasing position of this validation occurrence.
    /// - `path`: Path to the value for which the outcome was produced.
    /// - `outcome`: Validation result and any associated failure metadata.
    ///
    /// # Errors
    ///
    /// Returns an outcome shape error for malformed outcomes, duplicate
    /// prerequisite IDs, IDs not retained by this report, or an occurrence
    /// lower than the last successfully submitted occurrence. Such an error
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
        path: ValidationPath,
        outcome: ValidationOutcome,
    ) -> Result<RecordedOutcome, ValidationOutcomeError> {
        if self.last_recorded_occurrence.is_some_and(|last| occurrence < last) {
            return Err(ValidationOutcomeError::OutOfOrderOccurrence);
        }
        let result = match outcome {
            ValidationOutcome::Valid => Ok(RecordedOutcome::new(true, Vec::new())),
            ValidationOutcome::Invalid(violations) => {
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
                    self.failure_occurrences.push(occurrence);
                }
                if !complete {
                    self.mark_truncated();
                }
                Ok(RecordedOutcome::new(complete, failure_ids))
            }
            ValidationOutcome::Skipped {
                reason: SkipReason::MissingOptional,
                prerequisites,
            } => {
                if !prerequisites.is_empty() {
                    return Err(ValidationOutcomeError::UnexpectedPrerequisites);
                }
                if !self.has_skipped_capacity() {
                    self.mark_truncated();
                    Ok(RecordedOutcome::new(false, Vec::new()))
                } else {
                    self.skipped.push(SkippedValidation::missing_optional(occurrence, path));
                    Ok(RecordedOutcome::new(true, Vec::new()))
                }
            }
            ValidationOutcome::Skipped {
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
                    Ok(RecordedOutcome::new(false, Vec::new()))
                } else {
                    let skipped = SkippedValidation::failed_prerequisite(occurrence, path, prerequisites)?;
                    self.skipped.push(skipped);
                    Ok(RecordedOutcome::new(true, Vec::new()))
                }
            }
        };
        if result.is_ok() {
            self.last_recorded_occurrence = Some(occurrence);
        }
        result
    }

    /// Returns whether the recorded report contains no failures and is not
    /// marked incomplete. This evaluates recorded content only; it cannot
    /// establish that the caller ran every intended check.
    ///
    /// # Returns
    ///
    /// Returns `true` when there are no recorded violations, no occurrence was
    /// skipped because of a failed prerequisite, and the report was not
    /// truncated. A newly created empty report therefore returns `true`.
    ///
    /// Use this result as a basis for acceptance only after the caller has
    /// confirmed that the entire intended validation run completed
    /// successfully. A completed zero-rule plan may have an empty, valid
    /// report, but a newly created report alone does not prove completion.
    /// This method cannot know whether rules remain unsubmitted. If the caller
    /// stops validation early while checks may remain, it must call
    /// [`Self::mark_truncated`] so the report is marked incomplete. Reaching
    /// the report's capacity limit also marks it as truncated automatically.
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
    pub fn failure(&self, id: FailureId) -> Option<&Violation> {
        if id.report == self.report_id {
            self.violations.get(id.index)
        } else {
            None
        }
    }

    /// Returns the occurrence associated with a retained failure ID.
    ///
    /// Only failures retained in this report have an occurrence. IDs from a
    /// different report and failures rejected by the violation capacity return
    /// `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use qubit_validator::{ValidationOutcome, ValidationPath, ValidationReport};
    /// use qubit_validator::{ValidatorId, Violation, ViolationCode};
    ///
    /// let mut report = ValidationReport::new();
    /// let violation = Violation::new(
    ///     ValidatorId::new("example.required"),
    ///     ViolationCode::new("value.missing"),
    /// );
    /// let receipt = report.record_outcome(
    ///     2,
    ///     ValidationPath::root(),
    ///     ValidationOutcome::invalid(vec![violation])?,
    /// )?;
    /// let failure_id = receipt.failure_ids()[0];
    /// assert_eq!(report.failure_occurrence(failure_id), Some(2));
    /// # Ok::<(), qubit_validator::ValidationOutcomeError>(())
    /// ```
    pub fn failure_occurrence(&self, id: FailureId) -> Option<usize> {
        self.failure(id)?;
        self.failure_occurrences.get(id.index).copied()
    }

    /// Iterates over retained failures and their occurrences in report order.
    ///
    /// The iterator contains only failures retained within the configured
    /// violation capacity. Multiple failures from one outcome share an
    /// occurrence, and separate outcomes may use the same occurrence.
    #[must_use = "inspect all retained failures and their occurrences"]
    pub fn failures_with_occurrences(&self) -> impl Iterator<Item = (usize, &Violation)> + '_ {
        self.failure_occurrences.iter().copied().zip(self.violations.iter())
    }

    /// Returns all skipped occurrences in occurrence order.
    #[must_use = "the skipped occurrences should be inspected"]
    #[inline]
    pub fn skipped(&self) -> &[SkippedValidation] {
        &self.skipped
    }

    /// Returns whether this report is incomplete.
    ///
    /// # Returns
    ///
    /// Returns `true` when a collection limit rejected part of an outcome or
    /// the caller marked execution incomplete with [`Self::mark_truncated`].
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

    /// Returns the remaining capacity for retained original violations, or the
    /// largest possible count when unbounded. Prerequisite references do not
    /// consume this capacity.
    fn remaining_failure_capacity(&self) -> usize {
        self.limits
            .max_violations
            .map_or(usize::MAX, |limit| limit.saturating_sub(self.violations.len()))
    }

    /// Returns whether one more skipped occurrence fits the configured limit.
    fn has_skipped_capacity(&self) -> bool {
        self.limits.max_skipped.is_none_or(|limit| self.skipped.len() < limit)
    }
}

impl Default for ValidationReport {
    /// Creates an unbounded empty report.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ValidationReport {
    /// Formats counts and truncation state without exposing report contents.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ValidationReport")
            .field("violation_count", &self.failure_count())
            .field("skipped_count", &self.skipped.len())
            .field("truncated", &self.truncated)
            .finish()
    }
}

impl fmt::Display for ValidationReport {
    /// Formats counts and truncation state without exposing report contents.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "validation report: {} violation(s), {} skipped, truncated={}",
            self.failure_count(),
            self.skipped.len(),
            self.truncated,
        )
    }
}
