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
///
/// let mut report = ValidationReport::new();
/// let violation = Violation::new(
///     ValidatorId::new("example.non_empty"),
///     ViolationCode::new("text.empty"),
/// );
/// assert!(report.push_violation(violation));
/// assert!(!report.is_valid());
/// assert_eq!(report.violations().len(), 1);
/// ```
#[must_use]
pub struct ValidationReport {
    /// Accepted violations in occurrence order.
    violations: Vec<Violation>,
    /// Accepted skipped occurrences in occurrence order.
    skipped: Vec<SkippedValidation>,
    /// Whether configured limits prevented exhaustive collection.
    truncated: bool,
    /// Collection limits applied to both result categories.
    limits: ValidationLimits,
}

impl ValidationReport {
    /// Creates an empty report with explicit collection limits.
    #[inline]
    pub const fn with_limits(limits: ValidationLimits) -> Self {
        Self {
            violations: Vec::new(),
            skipped: Vec::new(),
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

    /// Appends a violation if the configured limit permits it.
    ///
    /// # Returns
    ///
    /// Returns `true` when the violation was stored and `false` when the limit
    /// rejected it and marked the report as truncated.
    pub fn push_violation(&mut self, violation: Violation) -> bool {
        if self
            .limits
            .max_violations
            .is_some_and(|limit| self.violations.len() >= limit)
        {
            self.mark_truncated();
            return false;
        }
        self.violations.push(violation);
        true
    }

    /// Records a skipped entry if the configured limit permits it.
    ///
    /// # Returns
    ///
    /// Returns `true` when the entry was stored and `false` when the limit
    /// rejected it and marked the report as truncated.
    pub fn push_skipped(&mut self, skipped: SkippedValidation) -> bool {
        if self.limits.max_skipped.is_some_and(|limit| self.skipped.len() >= limit) {
            self.mark_truncated();
            return false;
        }
        self.skipped.push(skipped);
        true
    }

    /// Marks that validation stopped before it was exhaustive.
    pub fn mark_truncated(&mut self) {
        self.truncated = true;
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
