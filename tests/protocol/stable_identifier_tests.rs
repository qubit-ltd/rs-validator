// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::sync::Arc;

use qubit_validator::BindError;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedValidator;
use qubit_validator::ValidationLimits;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorIdError;
use qubit_validator::ValidatorSignature;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationCodeError;
use qubit_validator::ViolationDraft;
use qubit_validator::ViolationParam;
use qubit_validator::prepare_text_validator;

const SENSITIVE_VALUE: &str = "raw-password-do-not-log";

#[derive(Debug, thiserror::Error)]
#[error("sensitive text rejected")]
struct SensitiveTextRejected;

struct RejectSensitiveText;

impl Validator<str> for RejectSensitiveText {
    type Error = SensitiveTextRejected;

    fn validate(&self, _: &str, _: &()) -> Result<(), Self::Error> {
        Err(SensitiveTextRejected)
    }
}

fn prepare_sensitive_text(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(prepare_text_validator(RejectSensitiveText, |_| {
        ViolationDraft::new(ViolationCode::new("credentials.rejected"))
            .with_param("retry_after", ViolationParam::Unsigned(3))
            .with_param("message_key", ViolationParam::Token("validation.credentials.rejected"))
            .with_param("minimum", ViolationParam::Signed(-2))
            .with_param("is_blocked", ViolationParam::Bool(true))
            .with_path(
                ValidationPath::root()
                    .with_field("credentials")
                    .with_index(0)
                    .with_map_entry(1)
                    .with_map_value(),
            )
    }))
}

static SENSITIVE_SIGNATURES: &[ValidatorSignature] =
    &[ValidatorSignature::new(InputType::Text, &[], prepare_sensitive_text)];
static SENSITIVE_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SENSITIVE_SIGNATURES);

fn violation() -> Violation {
    Violation::new(ValidatorId::new("test.protocol"), ViolationCode::new("test.rejected"))
}

#[test]
fn test_validator_id_protocol_boundaries_and_error_contract() {
    for (value, expected, display_context, debug_context) in [
        ("", ValidatorIdError::Empty, "empty", "Empty"),
        (
            ".leading",
            ValidatorIdError::EmptySegment,
            "empty segment",
            "EmptySegment",
        ),
        (
            "1leading",
            ValidatorIdError::InvalidSegment,
            "invalid segment",
            "InvalidSegment",
        ),
        (
            "middle-char",
            ValidatorIdError::InvalidSegment,
            "invalid segment",
            "InvalidSegment",
        ),
    ] {
        let error = ValidatorId::try_new(value).expect_err("invalid validator ID must be rejected");

        assert_eq!(error, expected);
        assert!(error.to_string().contains("validator ID"));
        assert!(error.to_string().contains(display_context));
        assert!(format!("{error:?}").contains(debug_context));
        assert!(std::error::Error::source(&error).is_none());
    }

    let shortest = ValidatorId::try_new("a").expect("single-letter validator ID is valid");
    let namespaced = ValidatorId::try_new("qubit.rules_2.text3").expect("namespaced validator ID is valid");

    assert_eq!(shortest.as_str(), "a");
    assert_eq!(namespaced.as_str(), "qubit.rules_2.text3");
    assert!(format!("{namespaced:?}").contains("ValidatorId"));
    assert!(format!("{namespaced:?}").contains(namespaced.as_str()));
}

#[test]
fn test_violation_code_protocol_boundaries_and_error_contract() {
    for (value, expected, display_context, debug_context) in [
        ("", ViolationCodeError::Empty, "empty", "Empty"),
        (
            ".leading",
            ViolationCodeError::EmptySegment,
            "empty segment",
            "EmptySegment",
        ),
        (
            "1leading",
            ViolationCodeError::InvalidSegment,
            "invalid segment",
            "InvalidSegment",
        ),
        (
            "middle-char",
            ViolationCodeError::InvalidSegment,
            "invalid segment",
            "InvalidSegment",
        ),
    ] {
        let error = ViolationCode::try_new(value).expect_err("invalid violation code must be rejected");

        assert_eq!(error, expected);
        assert!(error.to_string().contains("violation code"));
        assert!(error.to_string().contains(display_context));
        assert!(format!("{error:?}").contains(debug_context));
        assert!(std::error::Error::source(&error).is_none());
    }

    let shortest = ViolationCode::try_new("a").expect("single-letter violation code is valid");
    let namespaced = ViolationCode::try_new("text.length_2.minimum").expect("namespaced violation code is valid");

    assert_eq!(shortest.as_str(), "a");
    assert_eq!(namespaced.as_str(), "text.length_2.minimum");
    assert_eq!(namespaced.to_string(), namespaced.as_str());
    assert!(format!("{namespaced:?}").contains("ViolationCode"));
    assert!(format!("{namespaced:?}").contains(namespaced.as_str()));
}

#[test]
fn test_validator_id_and_violation_code_share_protocol_boundaries() {
    for (value, id_error, code_error) in [
        ("", ValidatorIdError::Empty, ViolationCodeError::Empty),
        (
            ".leading",
            ValidatorIdError::EmptySegment,
            ViolationCodeError::EmptySegment,
        ),
        (
            "trailing.",
            ValidatorIdError::EmptySegment,
            ViolationCodeError::EmptySegment,
        ),
        (
            "middle..empty",
            ValidatorIdError::EmptySegment,
            ViolationCodeError::EmptySegment,
        ),
        (
            "1leading",
            ValidatorIdError::InvalidSegment,
            ViolationCodeError::InvalidSegment,
        ),
        (
            "non-ascii-é",
            ValidatorIdError::InvalidSegment,
            ViolationCodeError::InvalidSegment,
        ),
    ] {
        assert_eq!(ValidatorId::try_new(value), Err(id_error));
        assert_eq!(ViolationCode::try_new(value), Err(code_error));
    }
    assert!(ValidatorId::try_new("one.two_2").is_ok());
    assert!(ViolationCode::try_new("one.two_2").is_ok());
}

#[test]
fn test_adapter_converts_draft_without_leaking_or_bypassing_report_limits() {
    let rule_id = ValidatorId::new("test.sensitive_text");
    let bound = SENSITIVE_DESCRIPTOR
        .bind_for(rule_id, InputType::Text, &[], &[])
        .expect("descriptor accepts its text signature");
    let outcome = bound
        .validate(
            ValidationValue::Text(SENSITIVE_VALUE),
            &BoundValidationContext::new(&[]),
        )
        .expect("validator execution succeeds");
    let ValidationOutcome::Invalid(mut violations) = outcome else {
        panic!("rejected sensitive text must produce an invalid outcome");
    };
    assert_eq!(violations.len(), 1);
    let final_violation = violations.pop().expect("invalid outcome contains the mapped violation");

    assert_eq!(final_violation.rule_id(), rule_id);
    assert_eq!(final_violation.code(), ViolationCode::new("credentials.rejected"));
    assert_eq!(
        final_violation.path(),
        &ValidationPath::root()
            .with_field("credentials")
            .with_index(0)
            .with_map_entry(1)
            .with_map_value(),
    );
    let params = final_violation
        .params()
        .iter()
        .map(|(name, value)| (*name, *value))
        .collect::<Vec<_>>();
    assert_eq!(
        params,
        [
            ("is_blocked", ViolationParam::Bool(true)),
            ("message_key", ViolationParam::Token("validation.credentials.rejected"),),
            ("minimum", ViolationParam::Signed(-2)),
            ("retry_after", ViolationParam::Unsigned(3)),
        ],
    );
    assert!(!format!("{final_violation:?}").contains(SENSITIVE_VALUE));
    assert!(!final_violation.to_string().contains(SENSITIVE_VALUE));

    let mut bounded_report = ValidationReport::with_limits(ValidationLimits {
        max_violations: Some(0),
        max_skipped: None,
    });
    assert!(
        !bounded_report
            .record_outcome(
                0,
                ValidationPath::root(),
                ValidationOutcome::invalid(vec![final_violation]).expect("a violation is present"),
            )
            .expect("capacity limits do not make the outcome malformed")
    );
    assert!(bounded_report.violations().is_empty());
    assert!(bounded_report.is_truncated());
}

#[test]
fn test_zero_report_limits_reject_entries_and_report_zero_counts() {
    let limits = ValidationLimits {
        max_violations: Some(0),
        max_skipped: Some(0),
    };
    let mut report = ValidationReport::with_limits(limits);

    assert!(
        !report
            .record_outcome(
                0,
                ValidationPath::root(),
                ValidationOutcome::invalid(vec![violation()]).expect("a violation is present"),
            )
            .expect("capacity limits do not make the outcome malformed")
    );
    assert!(
        !report
            .record_outcome(1, ValidationPath::root(), ValidationOutcome::missing_optional(),)
            .expect("missing optional is a valid outcome")
    );
    assert!(report.violations().is_empty());
    assert!(report.skipped().is_empty());
    assert!(report.is_truncated());
    assert!(!report.is_valid());
    assert_eq!(report.limits(), limits);

    let debug = format!("{report:?}");
    assert!(debug.contains("violation_count: 0"));
    assert!(debug.contains("skipped_count: 0"));
    assert!(debug.contains("truncated: true"));
    let display = report.to_string();
    assert!(display.contains("0 violation(s)"));
    assert!(display.contains("0 skipped"));
    assert!(display.contains("truncated=true"));
}

#[test]
fn test_report_limits_accept_exact_capacity_and_reject_one_over() {
    let mut violation_report = ValidationReport::with_limits(ValidationLimits {
        max_violations: Some(1),
        max_skipped: None,
    });
    assert!(
        violation_report
            .record_outcome(
                0,
                ValidationPath::root(),
                ValidationOutcome::invalid(vec![violation()]).expect("a violation is present"),
            )
            .expect("the first violation fits")
    );
    assert_eq!(violation_report.violations().len(), 1);
    assert!(!violation_report.is_truncated());
    assert!(
        !violation_report
            .record_outcome(
                1,
                ValidationPath::root(),
                ValidationOutcome::invalid(vec![violation()]).expect("a violation is present"),
            )
            .expect("capacity limits do not make the outcome malformed")
    );
    assert_eq!(violation_report.violations().len(), 1);
    assert!(violation_report.is_truncated());

    let mut skipped_report = ValidationReport::with_limits(ValidationLimits {
        max_violations: None,
        max_skipped: Some(1),
    });
    assert!(
        skipped_report
            .record_outcome(0, ValidationPath::root(), ValidationOutcome::missing_optional(),)
            .expect("first skipped occurrence fits")
    );
    assert_eq!(skipped_report.skipped().len(), 1);
    assert!(!skipped_report.is_truncated());
    assert!(
        !skipped_report
            .record_outcome(1, ValidationPath::root(), ValidationOutcome::missing_optional(),)
            .expect("capacity limits do not make the outcome malformed")
    );
    assert_eq!(skipped_report.skipped().len(), 1);
    assert!(skipped_report.is_truncated());
}
