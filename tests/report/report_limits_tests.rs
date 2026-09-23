// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::ValidationLimits;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;
use qubit_validator::ValidatorId;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;

fn violation() -> Violation {
    Violation::new(ValidatorId::new("test.rule"), ViolationCode::new("test.invalid"))
}

#[test]
fn test_report_limits_reject_and_mark_truncation() {
    let mut report = ValidationReport::with_limits(ValidationLimits {
        max_violations: Some(1),
        max_skipped: Some(0),
    });
    assert!(
        report
            .record_outcome(
                0,
                ValidationPath::root(),
                ValidationOutcome::invalid(vec![violation()]).expect("a violation is present")
            )
            .expect("first violation fits")
    );
    assert!(
        !report
            .record_outcome(
                1,
                ValidationPath::root(),
                ValidationOutcome::invalid(vec![violation()]).expect("a violation is present")
            )
            .expect("capacity truncation is an incomplete record")
    );
    assert!(
        !report
            .record_outcome(2, ValidationPath::root(), ValidationOutcome::missing_optional())
            .expect("missing optional is a valid outcome")
    );
    assert_eq!(report.violations().len(), 1);
    assert!(report.skipped().is_empty());
    assert!(report.is_truncated());
    assert!(!report.is_valid());
}

#[test]
fn test_report_without_limits_accepts_both_kinds() {
    let mut report = ValidationReport::new();
    assert!(
        report
            .record_outcome(
                0,
                ValidationPath::root(),
                ValidationOutcome::invalid(vec![violation()]).expect("a violation is present")
            )
            .expect("violation is accepted")
    );
    assert!(
        report
            .record_outcome(1, ValidationPath::root(), ValidationOutcome::missing_optional())
            .expect("skipped occurrence is accepted")
    );
    assert!(!report.is_truncated());
}

#[test]
fn test_caller_can_mark_early_stopping_as_truncated() {
    let mut report = ValidationReport::new();
    report.mark_truncated();
    assert!(report.is_truncated());
    assert!(!report.is_valid());
}
