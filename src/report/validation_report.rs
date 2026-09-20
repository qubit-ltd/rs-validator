// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Aggregated validation results.

use super::SkipReason;
use super::SkippedValidation;
use super::ValidationLimits;
use super::Violation;

/// A bounded collection of validation violations and skipped occurrences.
///
/// # Examples
///
/// ```
/// use qubit_validator::ValidationReport;
///
/// let report = ValidationReport::new();
/// assert!(report.is_valid());
/// ```
pub struct ValidationReport {
    violations: Vec<Violation>,
    skipped: Vec<SkippedValidation>,
    truncated: bool,
    limits: ValidationLimits,
}

impl ValidationReport {
    /// Creates an empty report with explicit collection limits.
    #[must_use]
    pub const fn with_limits(limits: ValidationLimits) -> Self {
        Self {
            violations: Vec::new(),
            skipped: Vec::new(),
            truncated: false,
            limits,
        }
    }

    /// Creates an empty report.
    #[must_use]
    pub const fn new() -> Self {
        Self::with_limits(ValidationLimits {
            max_violations: None,
            max_skipped: None,
        })
    }

    /// Appends a violation if the configured limit permits it.
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
    #[must_use]
    pub fn violations(&self) -> &[Violation] {
        &self.violations
    }

    /// Returns all skipped occurrences in occurrence order.
    #[must_use]
    pub fn skipped(&self) -> &[SkippedValidation] {
        &self.skipped
    }

    /// Returns whether validation was truncated.
    #[must_use]
    pub const fn is_truncated(&self) -> bool {
        self.truncated
    }

    /// Returns the configured collection limits.
    #[must_use]
    pub const fn limits(&self) -> ValidationLimits {
        self.limits
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ValidationReport {
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
