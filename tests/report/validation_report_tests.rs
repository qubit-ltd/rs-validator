// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::FailureId;
use qubit_validator::SkipReason;
use qubit_validator::ValidationLimits;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationOutcomeError;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;
use qubit_validator::ValidatorId;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;

fn violation(code: &'static str, field: &'static str) -> Violation {
    Violation::new(ValidatorId::new("test.rule"), ViolationCode::new(code))
        .with_path(ValidationPath::root().with_field(field))
}

fn record_failure(report: &mut ValidationReport, occurrence: usize) -> FailureId {
    report
        .record_outcome(
            occurrence,
            ValidationPath::root(),
            ValidationOutcome::invalid(vec![violation("test.invalid", "source")]).expect("one failure is valid"),
        )
        .expect("failure recording succeeds")
        .failure_ids()[0]
}

#[test]
fn test_partial_invalid_retains_only_prefix_with_resolvable_ids() {
    let mut report = ValidationReport::with_limits(ValidationLimits {
        max_violations: Some(2),
        max_skipped: None,
    });
    let prefix = ValidationPath::root().with_field("person").with_index(2);
    let receipt = report
        .record_outcome(
            4,
            prefix.clone(),
            ValidationOutcome::invalid(vec![
                violation("test.first", "name"),
                violation("test.second", "email"),
                violation("test.discarded", "phone"),
            ])
            .expect("invalid outcome has failures"),
        )
        .expect("limit exhaustion is an incomplete successful record");

    assert!(!receipt.complete());
    assert!(report.is_truncated());
    assert_eq!(report.failure_count(), 2);
    assert_eq!(receipt.failure_ids().len(), 2);
    for (index, failure_id) in receipt.failure_ids().iter().enumerate() {
        assert_eq!(report.failure(*failure_id), Some(&report.violations()[index]));
    }
    assert_eq!(report.violations()[0].code().as_str(), "test.first");
    assert_eq!(report.violations()[1].code().as_str(), "test.second");
    assert_eq!(report.violations()[0].path(), &prefix.clone().with_field("name"));
    assert_eq!(report.violations()[1].path(), &prefix.with_field("email"));
    assert!(
        !report
            .violations()
            .iter()
            .any(|item| item.code().as_str() == "test.discarded")
    );
}

#[test]
fn test_partial_invalid_receipt_contains_only_newly_retained_failure_ids() {
    let mut report = ValidationReport::with_limits(ValidationLimits {
        max_violations: Some(2),
        max_skipped: None,
    });
    let original = record_failure(&mut report, 0);
    let receipt = report
        .record_outcome(
            1,
            ValidationPath::root(),
            ValidationOutcome::invalid(vec![
                violation("test.retained", "one"),
                violation("test.discarded", "two"),
            ])
            .expect("invalid outcome has failures"),
        )
        .expect("partial recording succeeds");

    assert!(!receipt.complete());
    assert_eq!(receipt.failure_ids().len(), 1);
    assert_ne!(receipt.failure_ids()[0], original);
    assert_eq!(report.failure(receipt.failure_ids()[0]), Some(&report.violations()[1]));
    assert_eq!(report.failure_count(), 2);
}

#[test]
fn test_zero_violation_capacity_returns_no_failure_ids() {
    let mut report = ValidationReport::with_limits(ValidationLimits {
        max_violations: Some(0),
        max_skipped: None,
    });
    let receipt = report
        .record_outcome(
            0,
            ValidationPath::root(),
            ValidationOutcome::invalid(vec![violation("test.invalid", "field")])
                .expect("invalid outcome has a failure"),
        )
        .expect("capacity exhaustion is reported in the receipt");

    assert!(!receipt.complete());
    assert!(receipt.failure_ids().is_empty());
    assert!(report.violations().is_empty());
    assert!(report.is_truncated());
}

#[test]
fn test_malformed_skips_leave_report_state_and_occurrence_unchanged() {
    let mut report = ValidationReport::new();
    record_failure(&mut report, 0);
    report
        .record_outcome(1, ValidationPath::root(), ValidationOutcome::missing_optional())
        .expect("valid skip is retained");

    let missing_with_evidence = ValidationOutcome::Skipped {
        reason: SkipReason::MissingOptional,
        prerequisites: vec![
            report
                .record_outcome(
                    2,
                    ValidationPath::root(),
                    ValidationOutcome::invalid(vec![violation("test.second", "field")])
                        .expect("second failure is valid"),
                )
                .expect("second failure is retained")
                .failure_ids()[0],
        ],
    };
    let before_failure_count = report.failure_count();
    let before_skipped = report.skipped().to_vec();
    assert_eq!(
        report.record_outcome(4, ValidationPath::root(), missing_with_evidence),
        Err(ValidationOutcomeError::UnexpectedPrerequisites)
    );
    assert_eq!(report.failure_count(), before_failure_count);
    assert_eq!(report.skipped(), before_skipped);
    assert!(!report.is_truncated());

    assert_eq!(
        report.record_outcome(
            4,
            ValidationPath::root(),
            ValidationOutcome::Skipped {
                reason: SkipReason::FailedPrerequisite,
                prerequisites: Vec::new(),
            },
        ),
        Err(ValidationOutcomeError::EmptyPrerequisites)
    );
    assert_eq!(report.failure_count(), before_failure_count);
    assert_eq!(report.skipped(), before_skipped);
    assert!(!report.is_truncated());

    report
        .record_outcome(3, ValidationPath::root(), ValidationOutcome::valid())
        .expect("rejected outcomes do not advance the cursor");
}

#[test]
fn test_full_skip_capacity_truncates_and_advances_occurrence() {
    let mut report = ValidationReport::with_limits(ValidationLimits {
        max_violations: None,
        max_skipped: Some(0),
    });
    let failure_id = record_failure(&mut report, 0);
    let receipt = report
        .record_outcome(
            5,
            ValidationPath::root().with_field("target"),
            ValidationOutcome::failed_prerequisite(vec![failure_id]).expect("one prerequisite is well-formed"),
        )
        .expect("skip capacity exhaustion is an incomplete successful record");

    assert!(!receipt.complete());
    assert!(receipt.failure_ids().is_empty());
    assert_eq!(report.failure_count(), 1);
    assert!(report.skipped().is_empty());
    assert!(report.is_truncated());
    assert_eq!(
        report.record_outcome(4, ValidationPath::root(), ValidationOutcome::valid()),
        Err(ValidationOutcomeError::OutOfOrderOccurrence)
    );
}

#[test]
fn test_prerequisite_references_remain_valid_when_failure_capacity_is_full() {
    let mut report = ValidationReport::with_limits(ValidationLimits {
        max_violations: Some(1),
        max_skipped: None,
    });
    let failure_id = record_failure(&mut report, 0);
    let receipt = report
        .record_outcome(
            1,
            ValidationPath::root().with_field("dependent"),
            ValidationOutcome::failed_prerequisite(vec![failure_id]).expect("one prerequisite is well-formed"),
        )
        .expect("references use retained evidence without consuming capacity");

    assert!(receipt.complete());
    assert_eq!(report.failure_count(), 1);
    assert_eq!(report.skipped().len(), 1);
    assert_eq!(report.skipped()[0].prerequisites(), &[failure_id]);
    assert!(!report.is_truncated());
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
            ValidationOutcome::invalid(vec![violation("test.invalid", "source")]).expect("one source failure"),
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
            ValidationOutcome::invalid(vec![violation("test.invalid", "source")]).expect("first failure"),
        )
        .expect("first failure is retained")
        .failure_ids()[0];
    let second = report
        .record_outcome(
            1,
            ValidationPath::root(),
            ValidationOutcome::invalid(vec![violation("test.invalid", "source")]).expect("second failure"),
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
    assert_eq!(error, ValidationOutcomeError::UnknownPrerequisiteFailure);
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
            ValidationOutcome::invalid(vec![violation("test.invalid", "source")]).expect("source failure"),
        )
        .expect("source failure is retained")
        .failure_ids()[0];

    let duplicate = ValidationOutcome::failed_prerequisite(vec![failure_id, failure_id])
        .expect("shape validation is performed by the report");
    assert_eq!(
        report.record_outcome(1, ValidationPath::root(), duplicate),
        Err(ValidationOutcomeError::DuplicatePrerequisiteFailure)
    );
    let mut other_report = ValidationReport::new();
    let foreign = other_report
        .record_outcome(
            0,
            ValidationPath::root(),
            ValidationOutcome::invalid(vec![violation("test.invalid", "source")]).expect("foreign report failure"),
        )
        .expect("foreign failure is retained")
        .failure_ids()[0];
    let unknown = ValidationOutcome::failed_prerequisite(vec![foreign]).expect("one failure reference has valid shape");
    assert_eq!(
        report.record_outcome(2, ValidationPath::root(), unknown),
        Err(ValidationOutcomeError::UnknownPrerequisiteFailure)
    );
    assert!(report.skipped().is_empty());
    assert_eq!(report.failure_count(), 1);
}

#[test]
fn test_prerequisite_references_do_not_consume_failure_capacity() {
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
fn test_recording_returns_ids_for_original_failures_and_skips_reference_them() {
    let mut report = ValidationReport::new();
    let result = report
        .record_outcome(
            0,
            ValidationPath::root().with_field("source"),
            ValidationOutcome::invalid(vec![violation("test.invalid", "source")]).unwrap(),
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
fn test_malformed_invalid_outcome_is_rejected_without_mutation() {
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
fn test_report_rejects_out_of_order_occurrences_without_mutation() {
    let mut report = ValidationReport::new();
    report
        .record_outcome(3, ValidationPath::root(), ValidationOutcome::valid())
        .expect("first occurrence is accepted");

    assert_eq!(
        report.record_outcome(
            2,
            ValidationPath::root(),
            ValidationOutcome::invalid(vec![violation("test.invalid", "source")]).unwrap(),
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
            ValidationOutcome::invalid(vec![violation("test.invalid", "source")]).unwrap(),
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
