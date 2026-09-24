// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Aggregated validation results.

use super::SkipReason;
use super::SkippedValidation;
use super::ValidationLimits;
use super::ValidationOutcomeError;
use super::Violation;

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
/// assert!(report.record_outcome(
///     0,
///     ValidationPath::root(),
///     ValidationOutcome::invalid(vec![violation])?,
/// )?);
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
    /// Total retained violations, including nested prerequisite evidence.
    failure_count: usize,
    /// Whether configured limits prevented exhaustive collection.
    truncated: bool,
    /// Collection limits for all failures and skipped occurrences.
    limits: ValidationLimits,
}

impl ValidationReport {
    /// Creates an empty report with explicit collection limits.
    #[inline]
    pub const fn with_limits(limits: ValidationLimits) -> Self {
        Self {
            violations: Vec::new(),
            skipped: Vec::new(),
            failure_count: 0,
            truncated: false,
            limits,
        }
    }

    /// Creates an empty report.
    #[inline]
    pub const fn new() -> Self {
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
    /// their respective report limits. The violation limit counts both
    /// top-level violations and nested failed-prerequisite evidence. The
    /// supplied path prefixes each retained violation's relative path once.
    ///
    /// # Errors
    ///
    /// Returns an outcome shape error for an empty invalid result, an empty
    /// failed-prerequisite result, or a missing-optional result with
    /// prerequisite violations. Such an error leaves this report unchanged.
    ///
    /// # Returns
    ///
    /// Returns `true` when the whole outcome fits the configured limits and
    /// `false` when a limit rejects any part and marks the report truncated.
    /// An exhausted violation limit does not store a failed-prerequisite skip
    /// without evidence. A rejected skip consumes no violation capacity.
    pub fn record_outcome(
        &mut self,
        occurrence: usize,
        path: super::ValidationPath,
        outcome: crate::ValidationOutcome,
    ) -> Result<bool, ValidationOutcomeError> {
        match outcome {
            crate::ValidationOutcome::Valid => Ok(true),
            crate::ValidationOutcome::Invalid(violations) => {
                if violations.is_empty() {
                    return Err(ValidationOutcomeError::EmptyViolations);
                }
                let retained = violations.len().min(self.remaining_failure_capacity());
                let complete = retained == violations.len();
                for violation in violations.into_iter().take(retained) {
                    let full_path = path.concat(violation.path());
                    self.violations.push(violation.with_path(full_path));
                }
                self.failure_count += retained;
                if !complete {
                    self.mark_truncated();
                }
                Ok(complete)
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
                    return Ok(false);
                }
                self.skipped.push(SkippedValidation::missing_optional(occurrence, path));
                Ok(true)
            }
            crate::ValidationOutcome::Skipped {
                reason: SkipReason::FailedPrerequisite,
                prerequisites,
            } => {
                if prerequisites.is_empty() {
                    return Err(ValidationOutcomeError::EmptyPrerequisites);
                }
                if !self.has_skipped_capacity() {
                    self.mark_truncated();
                    return Ok(false);
                }
                let retained = prerequisites.len().min(self.remaining_failure_capacity());
                if retained == 0 {
                    self.mark_truncated();
                    return Ok(false);
                }
                let complete = retained == prerequisites.len();
                let evidence = prerequisites
                    .into_iter()
                    .take(retained)
                    .map(|violation| {
                        let full_path = path.concat(violation.path());
                        violation.with_path(full_path)
                    })
                    .collect();
                let skipped = SkippedValidation::failed_prerequisite(occurrence, path, evidence)?;
                self.skipped.push(skipped);
                self.failure_count += retained;
                if !complete {
                    self.mark_truncated();
                }
                Ok(complete)
            }
        }
    }

    /// Returns the remaining shared capacity for top-level and prerequisite
    /// violations, or the largest possible count when unbounded.
    fn remaining_failure_capacity(&self) -> usize {
        self.limits
            .max_violations
            .map_or(usize::MAX, |limit| limit.saturating_sub(self.failure_count))
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

    /// Returns the total retained failure count, including prerequisite
    /// violations nested in skipped entries.
    #[must_use]
    #[inline]
    pub const fn failure_count(&self) -> usize {
        self.failure_count
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
            .field("violation_count", &self.violations.len())
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
            self.violations.len(),
            self.skipped.len(),
            self.truncated,
        )
    }
}
