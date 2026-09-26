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
fn test_prerequisite_references_do_not_duplicate_or_consume_failure_capacity() {
    let mut report = ValidationReport::with_limits(ValidationLimits {
        max_violations: Some(1),
        max_skipped: None,
    });
    let recorded = report
        .record_outcome(
            0,
            ValidationPath::root().with_field("source"),
            ValidationOutcome::invalid(vec![violation()]).expect("one source failure"),
        )
        .expect("source failure is retained");
    let failure_id = recorded.failure_ids()[0];

    for occurrence in [1, 2] {
        let outcome = ValidationOutcome::failed_prerequisite(vec![failure_id]).expect("one prerequisite reference");
        assert!(
            report
                .record_outcome(occurrence, ValidationPath::root().with_index(occurrence), outcome)
                .expect("known prerequisite reference is valid")
                .complete()
        );
    }

    assert_eq!(report.failure_count(), 1);
    assert_eq!(report.failures().count(), 1);
    assert_eq!(report.skipped().len(), 2);
    assert_eq!(report.failure(failure_id), Some(&report.violations()[0]));
    assert!(!report.is_truncated());
}

#[test]
fn test_failure_ids_distinguish_equal_violations_and_reject_foreign_references() {
    let mut report = ValidationReport::new();
    let first = report
        .record_outcome(
            0,
            ValidationPath::root(),
            ValidationOutcome::invalid(vec![violation()]).expect("first failure"),
        )
        .expect("first failure is retained")
        .failure_ids()[0];
    let second = report
        .record_outcome(
            1,
            ValidationPath::root(),
            ValidationOutcome::invalid(vec![violation()]).expect("second failure"),
        )
        .expect("second failure is retained")
        .failure_ids()[0];
    assert_ne!(first, second);

    let mut other_report = ValidationReport::new();
    let before = other_report.skipped().len();
    let error = other_report
        .record_outcome(
            2,
            ValidationPath::root(),
            ValidationOutcome::failed_prerequisite(vec![first]).expect("one foreign reference"),
        )
        .expect_err("a failure ID belongs to its originating report");
    assert_eq!(
        error,
        qubit_validator::ValidationOutcomeError::UnknownPrerequisiteFailure
    );
    assert_eq!(other_report.skipped().len(), before);
    assert_eq!(other_report.failure_count(), 0);
}

#[test]
fn test_duplicate_or_missing_prerequisite_ids_leave_report_unchanged() {
    let mut report = ValidationReport::new();
    let failure_id = report
        .record_outcome(
            0,
            ValidationPath::root(),
            ValidationOutcome::invalid(vec![violation()]).expect("source failure"),
        )
        .expect("source failure is retained")
        .failure_ids()[0];

    let duplicate = ValidationOutcome::failed_prerequisite(vec![failure_id, failure_id])
        .expect("shape validation is performed by the report");
    assert_eq!(
        report.record_outcome(1, ValidationPath::root(), duplicate),
        Err(qubit_validator::ValidationOutcomeError::DuplicatePrerequisiteFailure)
    );
    let mut other_report = ValidationReport::new();
    let foreign = other_report
        .record_outcome(
            0,
            ValidationPath::root(),
            ValidationOutcome::invalid(vec![violation()]).expect("foreign report failure"),
        )
        .expect("foreign failure is retained")
        .failure_ids()[0];
    let unknown = ValidationOutcome::failed_prerequisite(vec![foreign]).expect("one failure reference has valid shape");
    assert_eq!(
        report.record_outcome(2, ValidationPath::root(), unknown),
        Err(qubit_validator::ValidationOutcomeError::UnknownPrerequisiteFailure)
    );
    assert!(report.skipped().is_empty());
    assert_eq!(report.failure_count(), 1);
}
