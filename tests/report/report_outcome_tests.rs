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
use qubit_validator::SkippedValidation;
use qubit_validator::ValidationLimits;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationOutcomeError;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;
use qubit_validator::ValidatorId;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;

fn create_violation(index: usize) -> Violation {
    Violation::new(
        ValidatorId::new("test.rule"),
        if index == 0 {
            ViolationCode::new("test.first")
        } else {
            ViolationCode::new("test.second")
        },
    )
}

fn create_draft() -> ViolationDraft {
    ViolationDraft::new(ViolationCode::new("test.invalid"))
}

#[test]
fn test_validation_path_concat_preserves_segment_order_and_inputs() {
    let prefix = ValidationPath::root().with_field("person").with_index(2);
    let relative = ValidationPath::root().with_field("name");
    let combined = prefix.concat(&relative);

    assert_eq!(
        combined.as_segments(),
        &[
            PathSegment::Field("person".into()),
            PathSegment::Index(2),
            PathSegment::Field("name".into()),
        ],
    );
    assert_eq!(prefix.as_segments().len(), 2);
    assert_eq!(relative.as_segments().len(), 1);
    assert_eq!(
        prefix.concat(&ValidationPath::root()).as_segments(),
        prefix.as_segments()
    );
    assert_eq!(
        ValidationPath::root().concat(&relative).as_segments(),
        relative.as_segments()
    );
}

#[test]
fn test_outcome_constructors_reject_empty_failure_data() {
    assert_eq!(
        ValidationOutcome::invalid(Vec::new()),
        Err(ValidationOutcomeError::EmptyViolations),
    );
    assert_eq!(
        PreparedOutcome::invalid(Vec::new()),
        Err(ValidationOutcomeError::EmptyViolations),
    );
    assert_eq!(
        ValidationOutcome::failed_prerequisite(Vec::new()),
        Err(ValidationOutcomeError::EmptyPrerequisites),
    );
    assert_eq!(
        PreparedOutcome::failed_prerequisite(Vec::new()),
        Err(ValidationOutcomeError::EmptyPrerequisites),
    );
}

#[test]
fn test_skipped_validation_constructors_keep_prerequisite_violations() {
    let missing = SkippedValidation::missing_optional(3, ValidationPath::root());
    assert_eq!(missing.reason(), SkipReason::MissingOptional);
    assert!(missing.prerequisites().is_empty());

    assert_eq!(
        SkippedValidation::failed_prerequisite(4, ValidationPath::root(), Vec::new()),
        Err(ValidationOutcomeError::EmptyPrerequisites),
    );

    let prerequisite = create_violation(0);
    let failed = SkippedValidation::failed_prerequisite(4, ValidationPath::root(), vec![prerequisite])
        .expect("a failed-prerequisite skip retains at least one violation");
    assert_eq!(failed.reason(), SkipReason::FailedPrerequisite);
    assert_eq!(failed.prerequisites().len(), 1);
}

#[test]
fn test_record_outcome_stores_invalid_violations_in_order() {
    let mut report = ValidationReport::new();
    let outcome = ValidationOutcome::invalid(vec![create_violation(0), create_violation(1)])
        .expect("invalid outcomes require violations");

    assert!(
        report
            .record_outcome(0, ValidationPath::root(), outcome)
            .expect("valid invalid outcome is recordable")
    );
    assert_eq!(report.violations()[0].code().as_str(), "test.first");
    assert_eq!(report.violations()[1].code().as_str(), "test.second");
    assert!(!report.is_valid());
    assert!(!report.is_truncated());
}

#[test]
fn test_record_outcome_prefixes_relative_invalid_paths_once() {
    let mut report = ValidationReport::new();
    let prefix = ValidationPath::root().with_field("person");
    let outcome = ValidationOutcome::invalid(vec![
        create_violation(0),
        create_violation(1).with_path(ValidationPath::root().with_field("name")),
    ])
    .expect("invalid outcomes require violations");

    assert!(report.record_outcome(0, prefix, outcome).expect("outcome fits"));
    assert_eq!(
        report.violations()[0].path().as_segments(),
        ValidationPath::root().with_field("person").as_segments(),
    );
    assert_eq!(
        report.violations()[1].path().as_segments(),
        ValidationPath::root()
            .with_field("person")
            .with_field("name")
            .as_segments(),
    );
}

#[test]
fn test_record_outcome_prefixes_relative_prerequisite_path_once() {
    let mut report = ValidationReport::new();
    let evidence = create_violation(0).with_path(ValidationPath::root().with_field("credential"));
    let outcome = ValidationOutcome::failed_prerequisite(vec![evidence]).expect("failed prerequisite has evidence");

    assert!(
        report
            .record_outcome(0, ValidationPath::root().with_field("person"), outcome)
            .expect("outcome fits")
    );
    assert_eq!(
        report.skipped()[0].prerequisites()[0].path().as_segments(),
        ValidationPath::root()
            .with_field("person")
            .with_field("credential")
            .as_segments(),
    );
}

#[test]
fn test_record_outcome_keeps_skipped_prerequisites_nested() {
    let prerequisite = create_violation(0);
    let outcome = ValidationOutcome::failed_prerequisite(vec![prerequisite]).expect("failed prerequisite has evidence");
    let mut report = ValidationReport::new();

    assert!(
        report
            .record_outcome(8, ValidationPath::root(), outcome)
            .expect("skipped outcome fits the report")
    );
    assert!(report.violations().is_empty());
    assert_eq!(report.skipped().len(), 1);
    assert_eq!(report.skipped()[0].occurrence(), 8);
    assert_eq!(report.skipped()[0].prerequisites().len(), 1);
    assert!(!report.is_valid());
}

#[test]
fn test_report_format_counts_failed_prerequisite_violations() {
    let mut report = ValidationReport::new();
    report
        .record_outcome(
            0,
            ValidationPath::root(),
            ValidationOutcome::failed_prerequisite(vec![create_violation(0)])
                .expect("failed prerequisite has evidence"),
        )
        .expect("report records the prerequisite failure");

    assert_eq!(report.failure_count(), 1);
    assert!(format!("{report:?}").contains("violation_count: 1"));
    assert!(report.to_string().contains("1 violation(s)"));
}

#[test]
fn test_record_outcome_valid_and_missing_optional_are_complete() {
    let mut report = ValidationReport::new();
    assert!(
        report
            .record_outcome(0, ValidationPath::root(), ValidationOutcome::valid())
            .expect("valid outcome is complete")
    );
    assert!(
        report
            .record_outcome(1, ValidationPath::root(), ValidationOutcome::missing_optional())
            .expect("missing optional outcome is complete")
    );
    assert_eq!(report.skipped().len(), 1);
    assert_eq!(report.skipped()[0].reason(), SkipReason::MissingOptional);
    assert!(report.is_valid());
}

#[test]
fn test_record_outcome_rejects_invalid_direct_enum_variants() {
    let mut report = ValidationReport::new();
    assert_eq!(
        report.record_outcome(0, ValidationPath::root(), ValidationOutcome::Invalid(Vec::new()),),
        Err(ValidationOutcomeError::EmptyViolations),
    );
    assert_eq!(
        report.record_outcome(
            1,
            ValidationPath::root(),
            ValidationOutcome::Skipped {
                reason: SkipReason::MissingOptional,
                prerequisites: vec![create_violation(0)],
            },
        ),
        Err(ValidationOutcomeError::UnexpectedPrerequisites),
    );
    assert_eq!(
        report.record_outcome(
            2,
            ValidationPath::root(),
            ValidationOutcome::Skipped {
                reason: SkipReason::FailedPrerequisite,
                prerequisites: Vec::new(),
            },
        ),
        Err(ValidationOutcomeError::EmptyPrerequisites),
    );
    assert!(report.violations().is_empty());
    assert!(report.skipped().is_empty());
}

#[test]
fn test_record_outcome_observes_violation_capacity_and_marks_truncation() {
    let exact = ValidationLimits {
        max_violations: Some(1),
        max_skipped: None,
    };
    let mut exact_report = ValidationReport::with_limits(exact);
    assert!(
        exact_report
            .record_outcome(
                0,
                ValidationPath::root(),
                ValidationOutcome::invalid(vec![create_violation(0)]).expect("one violation is present"),
            )
            .expect("an exact capacity fit is complete")
    );
    assert!(!exact_report.is_truncated());

    let mut overflow_report = ValidationReport::with_limits(exact);
    assert!(
        !overflow_report
            .record_outcome(
                0,
                ValidationPath::root(),
                ValidationOutcome::invalid(vec![create_violation(0), create_violation(1)])
                    .expect("two violations are present"),
            )
            .expect("capacity truncation is not an outcome shape error")
    );
    assert_eq!(overflow_report.violations().len(), 1);
    assert!(overflow_report.is_truncated());
}

#[test]
fn test_record_outcome_observes_zero_skipped_capacity() {
    let limits = ValidationLimits {
        max_violations: None,
        max_skipped: Some(0),
    };
    let mut report = ValidationReport::with_limits(limits);

    assert!(
        !report
            .record_outcome(0, ValidationPath::root(), ValidationOutcome::missing_optional())
            .expect("missing optional is a well-formed outcome")
    );
    assert!(report.skipped().is_empty());
    assert!(report.is_truncated());
}

#[test]
fn test_prepared_outcome_constructors_create_valid_outcomes() {
    assert_eq!(PreparedOutcome::valid(), PreparedOutcome::Valid);
    assert_eq!(
        PreparedOutcome::invalid(vec![create_draft()]),
        Ok(PreparedOutcome::Invalid(vec![create_draft()])),
    );
    assert_eq!(
        PreparedOutcome::missing_optional(),
        PreparedOutcome::Skipped {
            reason: SkipReason::MissingOptional,
            prerequisites: Vec::new(),
        },
    );
}
