// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::convert::Infallible;

use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::PathSegment;
use qubit_validator::PreparedOutcome;
use qubit_validator::SkipReason;
use qubit_validator::SkippedValidation;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;
use qubit_validator::Validator;
use qubit_validator::ValidatorId;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationParam;

struct NonEmpty;

impl Validator<str> for NonEmpty {
    type Error = Infallible;

    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        assert!(!value.is_empty());
        Ok(())
    }
}

#[test]
fn test_validator_uses_shared_immutable_context() {
    NonEmpty.validate("value", &()).expect("value is valid");
}

#[test]
fn test_violation_code_accepts_stable_dot_separated_names() {
    let code = ViolationCode::try_new("text.too_short").expect("valid code");

    assert_eq!(code.as_str(), "text.too_short");
    assert!(ViolationCode::try_new("text..too_short").is_err());
}

#[test]
fn test_path_builders_preserve_segments_without_map_keys() {
    let path = ValidationPath::root()
        .with_field("people")
        .with_index(2)
        .with_map_entry(4)
        .with_map_key();

    assert_eq!(
        path.as_segments(),
        &[
            PathSegment::Field("people".into()),
            PathSegment::Index(2),
            PathSegment::MapEntry(4),
            PathSegment::MapKey,
        ]
    );
    assert!(!format!("{path:?}").contains("people"));
    assert_eq!(path.render(), "people[2].<map-entry:4>.<map-key>");
}

#[test]
fn test_violation_and_report_keep_structured_safe_data() {
    let violation = Violation::new(
        ValidatorId::new("qubit.rules.text"),
        ViolationCode::new("text.too_short"),
    )
    .with_path(ValidationPath::root().with_field("name"))
    .with_param("min", ViolationParam::Unsigned(2));

    let mut report = ValidationReport::new();
    assert!(report.push_violation(violation));
    assert!(report.push_skipped(SkippedValidation::new(
        1,
        ValidationPath::root().with_field("optional"),
        SkipReason::MissingOptional,
    )));

    assert!(!report.is_valid());
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.skipped().len(), 1);
    assert!(!format!("{:?}", report).contains("name"));
    assert!(!format!("{:?}", report).contains("min"));
}

#[test]
fn test_failed_prerequisite_skip_is_invalid_and_outcome_is_explicit() {
    let mut report = ValidationReport::new();
    assert!(report.push_skipped(SkippedValidation::new(
        2,
        ValidationPath::root(),
        SkipReason::FailedPrerequisite,
    )));

    assert!(!report.is_valid());
    assert!(matches!(
        PreparedOutcome::Skipped {
            reason: SkipReason::FailedPrerequisite,
            prerequisites: vec![Violation::new(
                ValidatorId::new("qubit.rules.credential"),
                ViolationCode::new("credential.invalid"),
            )],
        },
        PreparedOutcome::Skipped { .. }
    ));
}

#[test]
fn test_execution_and_bind_errors_expose_kind_without_source_or_values() {
    let execution = ExecutionError::new(ExecutionErrorKind::ExternalFailure)
        .with_rule(ValidatorId::new("qubit.rules.remote"))
        .with_source(std::io::Error::other("secret input"));
    assert_eq!(execution.kind(), ExecutionErrorKind::ExternalFailure);
    assert!(!execution.to_string().contains("secret input"));
    assert!(!format!("{execution:?}").contains("secret input"));

    let bind = BindError::new(BindErrorKind::ParameterTypeMismatch)
        .with_parameter("minimum")
        .with_dependency("credential");
    assert_eq!(bind.kind(), BindErrorKind::ParameterTypeMismatch);
    assert!(bind.to_string().contains("parameter"));
    assert!(!bind.to_string().contains("secret input"));
}
