// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use std::convert::Infallible;
use std::sync::Arc;

use qubit_validator::BindError;
use qubit_validator::BoundValidationContext;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorSignature;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;
use qubit_validator::ViolationParam;
use qubit_validator::prepare_text_validator;

struct Minimum;

impl Validator<u32> for Minimum {
    type Error = Infallible;

    fn validate(&self, value: &u32, _: &()) -> Result<(), Self::Error> {
        assert!(*value >= 3);
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("text rejected")]
struct TextRejected;

struct TextRule;

impl Validator<str> for TextRule {
    type Error = TextRejected;

    fn validate(&self, _: &str, _: &()) -> Result<(), Self::Error> {
        Err(TextRejected)
    }
}

#[test]
fn typed_adapter_preserves_structured_violation_drafts() {
    let prepared = prepare_text_validator(TextRule, |_| {
        ViolationDraft::new(ViolationCode::new("test.rejected"))
            .with_param("bound", ViolationParam::Unsigned(3))
    });
    let outcome = prepared
        .validate(ValidationValue::Text("value"), &BoundValidationContext::new(&[]))
        .expect("adapter execution succeeds");
    assert!(matches!(outcome, PreparedOutcome::Invalid(violations) if violations.len() == 1));
}

#[test]
fn typed_validator_uses_an_immutable_context() {
    Minimum.validate(&5, &()).expect("value is valid");
}

struct Rejecting;

impl PreparedValidator for Rejecting {
    fn validate(
        &self,
        _: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        Ok(PreparedOutcome::Invalid(vec![ViolationDraft::new(ViolationCode::new(
            "test.rejected",
        ))]))
    }
}

fn prepare_rejecting(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(Rejecting))
}

static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare_rejecting)];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);

#[test]
fn prepared_descriptor_preserves_structured_rule_failures() {
    let bound = DESCRIPTOR
        .bind_for(ValidatorId::new("test.rejecting"), InputType::Text, &[], &[])
        .expect("valid descriptor");
    let outcome = bound
        .validate(ValidationValue::Text("value"), &BoundValidationContext::new(&[]))
        .expect("adapter execution succeeds");

    assert!(
        matches!(outcome, ValidationOutcome::Invalid(violations) if violations[0].code().as_str() == "test.rejected")
    );
}

#[test]
fn prepared_descriptor_rejects_wrong_input_shape() {
    let bound = DESCRIPTOR
        .bind_for(ValidatorId::new("test.rejecting"), InputType::Text, &[], &[])
        .expect("valid descriptor");
    let error = bound
        .validate(ValidationValue::Typed(&5_u32), &BoundValidationContext::new(&[]))
        .expect_err("wrong input shape must fail");

    assert_eq!(error.kind(), ExecutionErrorKind::InputTypeMismatch);
}
