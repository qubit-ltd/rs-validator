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
use qubit_validator::DependencySpec;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
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
use qubit_validator::prepare_text_domain_rule;
use qubit_validator::prepare_typed_domain_rule;

struct DomainRule;

#[derive(Debug)]
struct Rejected;
impl std::fmt::Display for Rejected {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("secret domain detail")
    }
}
impl std::error::Error for Rejected {}

impl Validator<str> for DomainRule {
    type Error = Rejected;
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        if value == "ok" { Ok(()) } else { Err(Rejected) }
    }
}

static DOMAIN_DEPENDENCIES: &[DependencySpec] = &[DependencySpec::new("required", InputType::Text, false)];

fn prepare_domain_with_dependency(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(prepare_text_domain_rule::<DomainRule, Rejected, (), _, _>(
        DOMAIN_DEPENDENCIES,
        DomainRule,
        |rule, value, _| Ok((rule.validate(value, &()), ())),
        |_, _| vec![ViolationDraft::new(ViolationCode::new("domain.rejected"))],
    ))
}

static DOMAIN_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    DOMAIN_DEPENDENCIES,
    prepare_domain_with_dependency,
)];
static DOMAIN_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(DOMAIN_SIGNATURES);

#[derive(Debug)]
struct TypedFailure;

impl std::fmt::Display for TypedFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("secret typed failure")
    }
}

impl std::error::Error for TypedFailure {}

fn prepare_typed_with_safe_configuration(
    _: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(prepare_typed_domain_rule::<u32, _, _, u32, _, _>(
        &[],
        (),
        |_, value, _| Ok((if *value == 1 { Ok(()) } else { Err(TypedFailure) }, 42)),
        |_, profile_id| {
            vec![
                ViolationDraft::new(ViolationCode::new("typed.rejected"))
                    .with_param("profile_id", ViolationParam::Unsigned(u128::from(*profile_id))),
            ]
        },
    ))
}

static TYPED_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::of::<u32>(),
    &[],
    prepare_typed_with_safe_configuration,
)];
static TYPED_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(TYPED_SIGNATURES);

#[test]
fn text_domain_adapter_maps_domain_error_into_safe_violation() {
    let prepared = prepare_text_domain_rule::<DomainRule, Rejected, &'static str, _, _>(
        &[],
        DomainRule,
        |rule, value, _| Ok((rule.validate(value, &()), "safe-profile")),
        |error, metadata| {
            assert_eq!(error.to_string(), "secret domain detail");
            assert_eq!(*metadata, "safe-profile");
            vec![ViolationDraft::new(ViolationCode::new("domain.rejected"))]
        },
    );
    assert!(run_text(prepared.as_ref(), "ok").is_ok());
    assert!(matches!(
        run_text(prepared.as_ref(), "bad"),
        Ok(PreparedOutcome::Invalid(_))
    ));
}

#[test]
fn domain_adapter_forwards_infrastructure_errors() {
    let prepared = prepare_text_domain_rule::<DomainRule, Rejected, (), _, _>(
        &[],
        DomainRule,
        |_, _, _| Err(ExecutionError::new(ExecutionErrorKind::DependencyTypeMismatch)),
        |_, _| vec![],
    );
    let error = run_text(prepared.as_ref(), "secret input").unwrap_err();
    assert_eq!(error.kind(), ExecutionErrorKind::DependencyTypeMismatch);
    assert!(!format!("{error:?}").contains("secret input"));
}

#[test]
fn domain_adapter_rejects_empty_invalid_drafts() {
    let prepared = prepare_text_domain_rule::<DomainRule, Rejected, (), _, _>(
        &[],
        DomainRule,
        |rule, value, _| Ok((rule.validate(value, &()), ())),
        |_, _| vec![],
    );
    let error = run_text(prepared.as_ref(), "bad").unwrap_err();
    assert_eq!(error.kind(), ExecutionErrorKind::AdapterContractViolation);
}

#[test]
fn domain_adapter_preserves_multiple_drafts_for_domain_failure() {
    let id = ValidatorId::new("test.domain");
    static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare_fixture)];
    let descriptor = ValidatorDescriptor::new(SIGNATURES);
    let bound = descriptor.bind(id, 0, &[]).unwrap();
    let outcome = bound
        .validate(ValidationValue::Text("bad"), &BoundValidationContext::new(&[]))
        .unwrap();
    assert!(matches!(outcome, ValidationOutcome::Invalid(ref violations)
        if violations.len() == 2 && violations[0].code().as_str() == "first"
            && violations[0].rule_id() == id && violations[1].code().as_str() == "second"));

    let prepared = prepare_text_domain_rule::<DomainRule, Rejected, (), _, _>(
        &[],
        DomainRule,
        |rule, value, _| Ok((rule.validate(value, &()), ())),
        |_, _| vec![ViolationDraft::new(ViolationCode::new("domain.rejected"))],
    );
    let rejected = run_text(prepared.as_ref(), "bad").expect("domain failure is an outcome");
    assert!(matches!(rejected, PreparedOutcome::Invalid(ref drafts) if !drafts.is_empty()));
}

fn prepare_fixture(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(prepare_text_domain_rule::<DomainRule, Rejected, (), _, _>(
        &[],
        DomainRule,
        |rule, value, _| Ok((rule.validate(value, &()), ())),
        |_, _| {
            vec![
                ViolationDraft::new(ViolationCode::new("first")),
                ViolationDraft::new(ViolationCode::new("second")),
            ]
        },
    ))
}

#[test]
fn test_prepared_domain_call_relies_on_bound_validator_for_dependency_contracts() {
    let prepared = prepare_domain_with_dependency(&[]).expect("domain adapter prepares");
    let direct = prepared
        .validate(ValidationValue::Text("ok"), &BoundValidationContext::new(&[]))
        .expect("direct prepared calls execute the supplied closure");
    assert_eq!(direct, PreparedOutcome::Valid);

    let bound = DOMAIN_DESCRIPTOR
        .bind(ValidatorId::new("test.domain.required"), 0, &[])
        .expect("descriptor prepares the matching dependency shape");
    let wrong_shape = bound
        .validate(ValidationValue::Text("ok"), &BoundValidationContext::new(&[]))
        .expect_err("bound validation rejects a context with the wrong slot count");
    assert_eq!(wrong_shape.kind(), ExecutionErrorKind::AdapterContractViolation);
    assert_eq!(wrong_shape.rule_id(), Some(ValidatorId::new("test.domain.required")));

    let missing = [ValidationValue::Missing];
    let path = ValidationPath::root().with_field("required");
    let paths = [path.clone()];
    let context = BoundValidationContext::new_with_paths(&missing, &paths).expect("one dependency path is valid");
    let error = bound
        .validate(ValidationValue::Text("ok"), &context)
        .expect_err("bound validation checks required dependencies before the adapter");
    assert_eq!(error.kind(), ExecutionErrorKind::MissingRequiredDependencyValue);
    assert_eq!(error.rule_id(), Some(ValidatorId::new("test.domain.required")));
    assert_eq!(error.dependency(), Some("required"));
    assert_eq!(error.path(), &path);
}

#[test]
fn typed_domain_adapter_passes_metadata_to_error_mapper() {
    let bound = TYPED_DESCRIPTOR
        .bind(ValidatorId::new("test.domain.typed"), 0, &[])
        .expect("typed domain rule binds");
    let context = BoundValidationContext::new(&[]);
    for input in [7_u32, 8] {
        let outcome = bound
            .validate(ValidationValue::Typed(&input), &context)
            .expect("domain failures are mapped into validation outcomes");
        assert!(matches!(&outcome, ValidationOutcome::Invalid(violations)
            if violations.len() == 1
                && violations[0].rule_id() == ValidatorId::new("test.domain.typed")
                && violations[0].code().as_str() == "typed.rejected"
                && violations[0].params().get("profile_id") == Some(&ViolationParam::Unsigned(42))
                && !violations[0].params().contains_key("value")));
        assert!(!format!("{outcome:?}").contains("secret typed failure"));
        assert!(!format!("{outcome:?}").contains(&input.to_string()));
    }
}

fn run_text(prepared: &dyn PreparedValidator, input: &str) -> Result<PreparedOutcome, ExecutionError> {
    prepared.validate(ValidationValue::Text(input), &BoundValidationContext::new(&[]))
}
