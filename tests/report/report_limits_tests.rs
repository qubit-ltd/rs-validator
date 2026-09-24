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

#[test]
fn test_failed_prerequisites_share_total_violation_capacity() {
    let mut report = ValidationReport::with_limits(ValidationLimits {
        max_violations: Some(1),
        max_skipped: None,
    });
    let outcome = ValidationOutcome::failed_prerequisite(vec![violation(), violation(), violation()])
        .expect("three prerequisites are valid evidence");

    assert!(
        !report
            .record_outcome(0, ValidationPath::root(), outcome)
            .expect("capacity truncation is not a shape error")
    );
    assert_eq!(report.skipped().len(), 1);
    assert_eq!(report.skipped()[0].prerequisites().len(), 1);
    assert_eq!(report.failure_count(), 1);
    assert!(report.is_truncated());
}

#[test]
fn test_exhausted_failure_capacity_does_not_store_empty_failed_skip() {
    let mut report = ValidationReport::with_limits(ValidationLimits {
        max_violations: Some(1),
        max_skipped: None,
    });
    assert!(
        report
            .record_outcome(
                0,
                ValidationPath::root(),
                ValidationOutcome::invalid(vec![violation()]).expect("one violation is valid"),
            )
            .expect("the first violation fits")
    );

    assert!(
        !report
            .record_outcome(
                1,
                ValidationPath::root(),
                ValidationOutcome::failed_prerequisite(vec![violation()]).expect("one prerequisite is valid"),
            )
            .expect("capacity truncation is not a shape error")
    );
    assert!(report.skipped().is_empty());
    assert_eq!(report.failure_count(), 1);
    assert!(report.is_truncated());
}

#[test]
fn test_zero_failure_capacity_discards_invalid_and_failed_prerequisites() {
    let mut report = ValidationReport::with_limits(ValidationLimits {
        max_violations: Some(0),
        max_skipped: None,
    });
    assert!(
        !report
            .record_outcome(
                0,
                ValidationPath::root(),
                ValidationOutcome::invalid(vec![violation()]).expect("one violation is valid"),
            )
            .expect("capacity truncation is not a shape error")
    );
    assert!(
        !report
            .record_outcome(
                1,
                ValidationPath::root(),
                ValidationOutcome::failed_prerequisite(vec![violation()]).expect("one prerequisite is valid"),
            )
            .expect("capacity truncation is not a shape error")
    );
    assert!(report.violations().is_empty());
    assert!(report.skipped().is_empty());
    assert_eq!(report.failure_count(), 0);
    assert!(report.is_truncated());
}

#[test]
fn test_zero_skipped_capacity_does_not_count_prerequisites() {
    let mut report = ValidationReport::with_limits(ValidationLimits {
        max_violations: Some(1),
        max_skipped: Some(0),
    });
    assert!(
        !report
            .record_outcome(
                0,
                ValidationPath::root(),
                ValidationOutcome::failed_prerequisite(vec![violation()]).expect("one prerequisite is valid"),
            )
            .expect("capacity truncation is not a shape error")
    );
    assert!(report.skipped().is_empty());
    assert_eq!(report.failure_count(), 0);
    assert!(report.is_truncated());
}

#[test]
fn test_invalid_and_prerequisites_share_total_violation_capacity() {
    let mut report = ValidationReport::with_limits(ValidationLimits {
        max_violations: Some(2),
        max_skipped: None,
    });
    assert!(
        report
            .record_outcome(
                0,
                ValidationPath::root(),
                ValidationOutcome::invalid(vec![violation()]).expect("one violation is valid"),
            )
            .expect("the first violation fits")
    );
    assert!(
        !report
            .record_outcome(
                1,
                ValidationPath::root(),
                ValidationOutcome::failed_prerequisite(vec![violation(), violation()])
                    .expect("two prerequisites are valid"),
            )
            .expect("capacity truncation is not a shape error")
    );
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.skipped()[0].prerequisites().len(), 1);
    assert_eq!(report.failure_count(), 2);
    assert!(report.is_truncated());
}
