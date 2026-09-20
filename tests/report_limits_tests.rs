// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use qubit_validator::SkipReason;
use qubit_validator::SkippedValidation;
use qubit_validator::ValidationLimits;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;
use qubit_validator::ValidatorId;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;

fn violation() -> Violation {
    Violation::new(ValidatorId::new("test.rule"), ViolationCode::new("test.invalid"))
}

fn skipped() -> SkippedValidation {
    SkippedValidation::new(
        1,
        ValidationPath::root().with_field("value"),
        SkipReason::MissingOptional,
    )
}

#[test]
fn report_limits_reject_and_mark_truncation() {
    let mut report = ValidationReport::with_limits(ValidationLimits {
        max_violations: Some(1),
        max_skipped: Some(0),
    });
    assert!(report.push_violation(violation()));
    assert!(!report.push_violation(violation()));
    assert!(!report.push_skipped(skipped()));
    assert_eq!(report.violations().len(), 1);
    assert!(report.skipped().is_empty());
    assert!(report.is_truncated());
    assert!(!report.is_valid());
}

#[test]
fn report_without_limits_accepts_both_kinds() {
    let mut report = ValidationReport::new();
    assert!(report.push_violation(violation()));
    assert!(report.push_skipped(skipped()));
    assert!(!report.is_truncated());
}
