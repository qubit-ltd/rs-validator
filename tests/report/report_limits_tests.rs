// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::FailureId;
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
fn record_failure(report: &mut ValidationReport, occurrence: usize) -> FailureId {
    report
        .record_outcome(
            occurrence,
            ValidationPath::root(),
            ValidationOutcome::invalid(vec![violation()]).unwrap(),
        )
        .unwrap()
        .failure_ids()[0]
}
#[test]
fn prerequisite_references_do_not_consume_failure_capacity() {
    let mut report = ValidationReport::with_limits(ValidationLimits {
        max_violations: Some(1),
        max_skipped: None,
    });
    let id = record_failure(&mut report, 0);
    for occurrence in 1..=2 {
        let receipt = report
            .record_outcome(
                occurrence,
                ValidationPath::root(),
                ValidationOutcome::failed_prerequisite(vec![id]).unwrap(),
            )
            .unwrap();
        assert!(receipt.complete());
    }
    assert_eq!(report.failure_count(), 1);
    assert_eq!(report.failures().count(), 1);
    assert_eq!(report.skipped().len(), 2);
    assert!(!report.is_truncated());
}
#[test]
fn limits_retain_partial_original_failure_ids_and_mark_truncation() {
    let mut report = ValidationReport::with_limits(ValidationLimits {
        max_violations: Some(1),
        max_skipped: Some(0),
    });
    let first = report
        .record_outcome(
            0,
            ValidationPath::root(),
            ValidationOutcome::invalid(vec![violation()]).unwrap(),
        )
        .unwrap();
    assert!(first.complete());
    assert_eq!(first.failure_ids().len(), 1);
    let second = report
        .record_outcome(
            1,
            ValidationPath::root(),
            ValidationOutcome::invalid(vec![violation()]).unwrap(),
        )
        .unwrap();
    assert!(!second.complete());
    assert!(second.failure_ids().is_empty());
    assert!(report.is_truncated());
    assert!(
        !report
            .record_outcome(2, ValidationPath::root(), ValidationOutcome::missing_optional())
            .unwrap()
            .complete()
    );
}
#[test]
fn zero_failure_capacity_cannot_create_a_prerequisite_reference() {
    let mut report = ValidationReport::with_limits(ValidationLimits {
        max_violations: Some(0),
        max_skipped: None,
    });
    let receipt = report
        .record_outcome(
            0,
            ValidationPath::root(),
            ValidationOutcome::invalid(vec![violation()]).unwrap(),
        )
        .unwrap();
    assert!(receipt.failure_ids().is_empty());
    assert!(!receipt.complete());
    assert_eq!(report.failure_count(), 0);
}
