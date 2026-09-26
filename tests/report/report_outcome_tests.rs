// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::PathSegment;
use qubit_validator::PreparedOutcome;
use qubit_validator::SkipReason;
use qubit_validator::ValidationLimits;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationOutcomeError;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;
use qubit_validator::ValidatorId;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;
fn violation() -> Violation {
    Violation::new(ValidatorId::new("test.rule"), ViolationCode::new("test.invalid"))
}
#[test]
fn path_concat_preserves_segment_order() {
    let prefix = ValidationPath::root().with_field("person").with_index(2);
    let relative = ValidationPath::root().with_field("name");
    assert_eq!(
        prefix.concat(&relative).as_segments(),
        &[
            PathSegment::Field("person"),
            PathSegment::Index(2),
            PathSegment::Field("name")
        ]
    );
}
#[test]
fn constructors_reject_empty_failure_data() {
    assert_eq!(
        ValidationOutcome::invalid(Vec::new()),
        Err(ValidationOutcomeError::EmptyViolations)
    );
    assert_eq!(
        PreparedOutcome::invalid(Vec::new()),
        Err(ValidationOutcomeError::EmptyViolations)
    );
    assert_eq!(
        ValidationOutcome::failed_prerequisite(Vec::new()),
        Err(ValidationOutcomeError::EmptyPrerequisites)
    );
}
#[test]
fn recording_returns_ids_for_original_failures_and_skips_reference_them() {
    let mut report = ValidationReport::new();
    let result = report
        .record_outcome(
            0,
            ValidationPath::root().with_field("source"),
            ValidationOutcome::invalid(vec![violation()]).unwrap(),
        )
        .unwrap();
    let id = result.failure_ids()[0];
    assert_eq!(report.failure(id), Some(&report.violations()[0]));
    let skipped = report
        .record_outcome(
            1,
            ValidationPath::root().with_field("target"),
            ValidationOutcome::failed_prerequisite(vec![id]).unwrap(),
        )
        .unwrap();
    assert!(skipped.complete());
    assert_eq!(report.failure_count(), 1);
    assert_eq!(report.failures().count(), 1);
    assert_eq!(report.skipped()[0].prerequisites(), &[id]);
    assert!(!report.is_valid());
}
#[test]
fn malformed_outcomes_are_rejected_without_mutation() {
    let mut report = ValidationReport::new();
    assert_eq!(
        report.record_outcome(0, ValidationPath::root(), ValidationOutcome::Invalid(Vec::new())),
        Err(ValidationOutcomeError::EmptyViolations)
    );
    let receipt = report
        .record_outcome(0, ValidationPath::root(), ValidationOutcome::missing_optional())
        .unwrap();
    assert!(receipt.complete());
    assert_eq!(report.skipped()[0].reason(), SkipReason::MissingOptional);
}
#[test]
fn prepared_outcome_preserves_drafts() {
    let draft = ViolationDraft::new(ViolationCode::new("test.invalid"));
    assert!(matches!(
        PreparedOutcome::invalid(vec![draft]),
        Ok(PreparedOutcome::Invalid(_))
    ));
}

#[test]
fn test_report_rejects_out_of_order_occurrences_without_mutation() {
    let mut report = ValidationReport::new();
    report
        .record_outcome(3, ValidationPath::root(), ValidationOutcome::valid())
        .expect("first occurrence is accepted");

    assert_eq!(
        report.record_outcome(
            2,
            ValidationPath::root(),
            ValidationOutcome::invalid(vec![violation()]).unwrap(),
        ),
        Err(ValidationOutcomeError::OutOfOrderOccurrence),
    );
    assert_eq!(report.failure_count(), 0);
    assert!(report.skipped().is_empty());
    assert!(!report.is_truncated());

    report
        .record_outcome(
            3,
            ValidationPath::root(),
            ValidationOutcome::invalid(vec![violation()]).unwrap(),
        )
        .expect("the same occurrence may be recorded more than once");
    assert_eq!(report.failure_count(), 1);
}

#[test]
fn test_failed_record_does_not_advance_report_occurrence() {
    let mut report = ValidationReport::new();
    assert_eq!(
        report.record_outcome(5, ValidationPath::root(), ValidationOutcome::Invalid(Vec::new())),
        Err(ValidationOutcomeError::EmptyViolations),
    );
    report
        .record_outcome(4, ValidationPath::root(), ValidationOutcome::missing_optional())
        .expect("a rejected outcome must not advance the occurrence cursor");
    assert_eq!(report.skipped().len(), 1);
}

#[test]
fn test_incomplete_record_advances_report_occurrence() {
    let mut report = ValidationReport::with_limits(ValidationLimits {
        max_violations: None,
        max_skipped: Some(0),
    });
    let receipt = report
        .record_outcome(5, ValidationPath::root(), ValidationOutcome::missing_optional())
        .expect("limit exhaustion is an incomplete but successful record");
    assert!(!receipt.complete());

    assert_eq!(
        report.record_outcome(4, ValidationPath::root(), ValidationOutcome::valid()),
        Err(ValidationOutcomeError::OutOfOrderOccurrence),
    );
    assert!(report.is_truncated());
}
