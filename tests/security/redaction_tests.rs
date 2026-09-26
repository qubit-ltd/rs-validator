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
use qubit_validator::BoundValidationContext;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::InputType;
use qubit_validator::PathSegment;
use qubit_validator::SkipReason;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorId;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;
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
fn test_debug_surfaces_redact_values_and_show_safe_shapes() {
    let secret = "sensitive-input";
    let value = ValidationValue::Text(secret);
    let values = [value];
    let context = BoundValidationContext::new(&values);
    let draft =
        ViolationDraft::new(ViolationCode::new("text.invalid")).with_path(ValidationPath::root().with_field("value"));

    for debug in [format!("{value:?}"), format!("{context:?}"), format!("{draft:?}")] {
        assert!(!debug.contains(secret));
    }
    assert_eq!(InputType::Text, value.input_type().expect("text has an input type"));

    let dynamic_entry = ValidationPath::root().with_map_entry(0).with_map_value();
    assert_eq!(dynamic_entry.render(), ".<map-entry:0>.<map-value>");
    assert!(!dynamic_entry.render().contains(secret));
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
            PathSegment::Field("people"),
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
    assert!(
        report
            .record_outcome(
                0,
                ValidationPath::root(),
                ValidationOutcome::invalid(vec![violation]).expect("a violation is present")
            )
            .expect("violation is accepted")
            .complete()
    );
    assert!(
        report
            .record_outcome(
                1,
                ValidationPath::root().with_field("optional"),
                ValidationOutcome::missing_optional()
            )
            .expect("missing optional is a valid outcome")
            .complete()
    );

    assert!(!report.is_valid());
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.skipped().len(), 1);
    assert!(!format!("{:?}", report).contains("name"));
    assert!(!format!("{:?}", report).contains("min"));
}

#[test]
fn test_failed_prerequisite_skip_is_invalid_and_outcome_is_explicit() {
    let mut report = ValidationReport::new();
    let source = report
        .record_outcome(
            1,
            ValidationPath::root().with_field("credential"),
            ValidationOutcome::invalid(vec![Violation::new(
                ValidatorId::new("qubit.rules.credential"),
                ViolationCode::new("credential.invalid"),
            )])
            .expect("a source violation is present"),
        )
        .expect("source failure is retained");
    let id = source.failure_ids()[0];
    report
        .record_outcome(
            2,
            ValidationPath::root(),
            ValidationOutcome::failed_prerequisite(vec![id]).expect("a failed prerequisite is present"),
        )
        .expect("failed prerequisite is stored");

    assert!(!report.is_valid());
    assert_eq!(report.skipped()[0].reason(), SkipReason::FailedPrerequisite);
    assert_eq!(report.skipped()[0].prerequisites(), &[id]);
}

#[test]
fn test_execution_and_bind_errors_expose_kind_without_source_or_values() {
    let execution =
        ExecutionError::new(ExecutionErrorKind::ExternalFailure).with_rule(ValidatorId::new("qubit.rules.remote"));
    assert_eq!(execution.kind(), ExecutionErrorKind::ExternalFailure);
    assert!(std::error::Error::source(&execution).is_none());
    assert!(!execution.to_string().contains("secret input"));
    assert!(!format!("{execution:?}").contains("secret input"));

    let bind = BindError::new(BindErrorKind::ParameterTypeMismatch)
        .with_parameter("minimum")
        .with_dependency("credential");
    assert_eq!(bind.kind(), BindErrorKind::ParameterTypeMismatch);
    assert!(bind.to_string().contains("parameter"));
    assert!(!bind.to_string().contains("secret input"));
}

#[derive(Debug)]
struct SensitiveCause {
    message: &'static str,
}

impl std::fmt::Display for SensitiveCause {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message)
    }
}

impl std::error::Error for SensitiveCause {}

#[test]
fn test_trusted_execution_source_is_explicit_and_ordinary_formats_redact_it() {
    let error = ExecutionError::new(ExecutionErrorKind::ExternalFailure).with_trusted_source(SensitiveCause {
        message: "source-secret",
    });
    let source = error.trusted_source().expect("trusted cause is retained");

    assert_eq!(source.to_string(), "source-secret");
    assert!(source.downcast_ref::<SensitiveCause>().is_some());
    assert!(!format!("{error:?}").contains("source-secret"));
    assert!(!error.to_string().contains("source-secret"));
    assert!(std::error::Error::source(&error).is_none());

    let without_source = ExecutionError::new(ExecutionErrorKind::ExternalFailure);
    assert!(without_source.trusted_source().is_none());
}
