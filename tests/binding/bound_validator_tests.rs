// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::convert::Infallible;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_validator::BindError;
use qubit_validator::BoundValidationContext;
use qubit_validator::BoundValidator;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PathSegment;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::SkipReason;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationPath;
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
fn test_typed_adapter_preserves_structured_violation_drafts() {
    let prepared = prepare_text_validator(TextRule, |_| {
        ViolationDraft::new(ViolationCode::new("test.rejected")).with_param("bound", ViolationParam::Unsigned(3))
    });
    let outcome = prepared
        .validate(ValidationValue::Text("value"), &BoundValidationContext::new(&[]))
        .expect("adapter execution succeeds");
    assert!(matches!(outcome, PreparedOutcome::Invalid(violations) if violations.len() == 1));
}

#[test]
fn test_typed_validator_uses_an_immutable_context() {
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
fn test_prepared_descriptor_preserves_structured_rule_failures() {
    let bound = DESCRIPTOR
        .bind_for(ValidatorId::new("test.rejecting"), InputType::Text, &[])
        .expect("valid descriptor");
    let outcome = bound
        .validate(ValidationValue::Text("value"), &BoundValidationContext::new(&[]))
        .expect("adapter execution succeeds");

    assert!(
        matches!(outcome, ValidationOutcome::Invalid(violations) if violations[0].code().as_str() == "test.rejected")
    );
}

#[test]
fn test_prepared_descriptor_rejects_wrong_input_shape() {
    let bound = DESCRIPTOR
        .bind_for(ValidatorId::new("test.rejecting"), InputType::Text, &[])
        .expect("valid descriptor");
    let error = bound
        .validate(ValidationValue::Typed(&5_u32), &BoundValidationContext::new(&[]))
        .expect_err("wrong input shape must fail");

    assert_eq!(error.kind(), ExecutionErrorKind::InputTypeMismatch);
}

#[test]
fn test_prepared_outcome_binds_rule_identity_and_preserves_draft_metadata() {
    let rule_id = ValidatorId::new("test.bound_rule");
    let outcome = PreparedOutcome::Invalid(vec![
        ViolationDraft::new(ViolationCode::new("test.rejected"))
            .with_path(ValidationPath::root().with_field("name"))
            .with_param("minimum", ViolationParam::Unsigned(3)),
    ])
    .into_bound(rule_id)
    .expect("one draft makes a valid bound outcome");

    let ValidationOutcome::Invalid(violations) = outcome else {
        panic!("an invalid draft must remain invalid");
    };
    assert_eq!(violations[0].rule_id(), rule_id);
    assert_eq!(violations[0].code().as_str(), "test.rejected");
    assert!(matches!(
        violations[0].path().as_segments(),
        [PathSegment::Field(name)] if *name == "name"
    ));
    assert_eq!(violations[0].params()["minimum"], ViolationParam::Unsigned(3));
}

#[test]
fn test_validation_outcome_constructs_skips_with_consistent_evidence() {
    let missing = ValidationOutcome::missing_optional();
    assert!(matches!(missing, ValidationOutcome::Skipped {
        reason: SkipReason::MissingOptional,
        prerequisites,
    } if prerequisites.is_empty()));

    assert!(matches!(
        ValidationOutcome::failed_prerequisite(Vec::new()),
        Err(qubit_validator::ValidationOutcomeError::EmptyPrerequisites)
    ));
}

#[test]
fn test_validation_outcome_rejects_failed_prerequisite_without_evidence() {
    assert!(matches!(
        ValidationOutcome::failed_prerequisite(Vec::new()),
        Err(qubit_validator::ValidationOutcomeError::EmptyPrerequisites)
    ));
    assert!(matches!(
        PreparedOutcome::Invalid(Vec::new()).into_bound(ValidatorId::new("test.empty")),
        Err(qubit_validator::ValidationOutcomeError::EmptyViolations)
    ));
}

struct CountingPrepared(AtomicUsize);

impl PreparedValidator for CountingPrepared {
    fn validate(
        &self,
        _: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Ok(PreparedOutcome::valid())
    }
}

#[test]
fn test_prepared_bound_validator_checks_input_and_empty_dependencies() {
    let rule_id = ValidatorId::new("test.model_rule");
    let prepared = Arc::new(CountingPrepared(AtomicUsize::new(0)));
    let bound = BoundValidator::from_prepared::<u32>(rule_id, prepared.clone());
    let empty_context = BoundValidationContext::new(&[]);

    assert_eq!(bound.input_type(), InputType::of::<u32>());
    assert!(bound.dependency_specs().is_empty());
    assert!(matches!(
        bound.validate(ValidationValue::Typed(&42_u32), &empty_context),
        Ok(ValidationOutcome::Valid),
    ));
    assert_eq!(prepared.0.load(Ordering::Relaxed), 1);

    let wrong_type = bound
        .validate(ValidationValue::Typed(&42_i32), &empty_context)
        .expect_err("the exact concrete input type is enforced");
    assert_eq!(wrong_type.kind(), ExecutionErrorKind::InputTypeMismatch);
    assert_eq!(wrong_type.rule_id(), Some(rule_id));

    let extra_values = [ValidationValue::Text("unexpected")];
    let extra_context = BoundValidationContext::new(&extra_values);
    let wrong_context = bound
        .validate(ValidationValue::Typed(&42_u32), &extra_context)
        .expect_err("a zero-dependency rule rejects extra slots");
    assert_eq!(wrong_context.kind(), ExecutionErrorKind::AdapterContractViolation);
    assert_eq!(wrong_context.rule_id(), Some(rule_id));
    assert_eq!(prepared.0.load(Ordering::Relaxed), 1);
}

struct FixedOutcome {
    make: fn() -> PreparedOutcome,
}

impl PreparedValidator for FixedOutcome {
    fn validate(
        &self,
        _: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        Ok((self.make)())
    }
}

fn prepare_empty_invalid(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(FixedOutcome {
        make: || PreparedOutcome::Invalid(Vec::new()),
    }))
}

static EMPTY_INVALID_SIGNATURES: &[ValidatorSignature] =
    &[ValidatorSignature::new(InputType::Text, &[], prepare_empty_invalid)];

#[test]
fn test_bound_validate_rejects_empty_prepared_outcome_with_bound_rule() {
    let rule_id = ValidatorId::new("test.bound_contract");
    let bound = ValidatorDescriptor::new(EMPTY_INVALID_SIGNATURES)
        .bind(rule_id, 0, &[])
        .expect("valid test descriptor");
    let error = bound
        .validate(ValidationValue::Text("secret input"), &BoundValidationContext::new(&[]))
        .expect_err("an empty invalid result violates the prepared contract");
    assert_eq!(error.kind(), ExecutionErrorKind::AdapterContractViolation);
    assert_eq!(error.rule_id(), Some(rule_id));
    assert!(!format!("{error:?}").contains("secret input"));
    assert!(!error.to_string().contains("secret input"));
}

fn prepare_draft_with_metadata(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(FixedOutcome {
        make: || {
            PreparedOutcome::Invalid(vec![
                ViolationDraft::new(ViolationCode::new("test.rejected"))
                    .with_path(ValidationPath::root().with_field("name"))
                    .with_param("minimum", ViolationParam::Unsigned(3)),
            ])
        },
    }))
}

static METADATA_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    &[],
    prepare_draft_with_metadata,
)];

#[test]
fn test_bound_validate_binds_draft_metadata() {
    let rule_id = ValidatorId::new("test.bound_metadata");
    let bound = ValidatorDescriptor::new(METADATA_SIGNATURES)
        .bind(rule_id, 0, &[])
        .expect("valid test descriptor");
    let outcome = bound
        .validate(ValidationValue::Text("secret input"), &BoundValidationContext::new(&[]))
        .expect("draft outcome is valid");
    let ValidationOutcome::Invalid(violations) = outcome else {
        panic!("draft must bind to invalid outcome");
    };
    assert_eq!(violations.len(), 1);
    let violation = &violations[0];
    assert_eq!(violation.rule_id(), rule_id);
    assert_eq!(violation.code().as_str(), "test.rejected");
    assert!(matches!(violation.path().as_segments(), [PathSegment::Field(name)] if *name == "name"));
    assert_eq!(violation.params()["minimum"], ViolationParam::Unsigned(3));
    assert!(!format!("{violation:?}").contains("secret input"));
    assert!(!violation.to_string().contains("secret input"));
}
