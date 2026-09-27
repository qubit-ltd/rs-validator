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
use qubit_validator::DomainErrorDisposition;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
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

#[test]
fn text_domain_adapter_maps_domain_error_into_safe_violation() {
    let prepared = prepare_text_domain_rule::<DomainRule, Rejected, &'static str, _, _>(
        &[],
        DomainRule,
        |rule, value, _| Ok((rule.validate(value, &()), "safe-profile")),
        |error, metadata| {
            assert_eq!(error.to_string(), "secret domain detail");
            assert_eq!(*metadata, "safe-profile");
            DomainErrorDisposition::Invalid(vec![ViolationDraft::new(ViolationCode::new("domain.rejected"))])
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
        |_, _| DomainErrorDisposition::Valid,
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
        |_, _| DomainErrorDisposition::Invalid(vec![]),
    );
    let error = run_text(prepared.as_ref(), "bad").unwrap_err();
    assert_eq!(error.kind(), ExecutionErrorKind::AdapterContractViolation);
}

#[test]
fn domain_adapter_preserves_multiple_drafts_and_ignores_domain_failure() {
    let id = ValidatorId::new("test.domain");
    static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare_fixture)];
    let descriptor = ValidatorDescriptor::new(SIGNATURES);
    let bound = descriptor.bind(id, 0, &[]).unwrap();
    let outcome = bound
        .validate(ValidationValue::Text("bad"), &BoundValidationContext::new(&[]))
        .unwrap();
    assert!(
        matches!(outcome, qubit_validator::ValidationOutcome::Invalid(ref violations)
        if violations.len() == 2 && violations[0].code().as_str() == "first"
            && violations[0].rule_id() == id && violations[1].code().as_str() == "second")
    );

    let ignored = prepare_text_domain_rule::<DomainRule, Rejected, (), _, _>(
        &[],
        DomainRule,
        |rule, value, _| Ok((rule.validate(value, &()), ())),
        |_, _| DomainErrorDisposition::Valid,
    );
    assert!(matches!(run_text(ignored.as_ref(), "bad"), Ok(PreparedOutcome::Valid)));
}

fn prepare_fixture(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(prepare_text_domain_rule::<DomainRule, Rejected, (), _, _>(
        &[],
        DomainRule,
        |rule, value, _| Ok((rule.validate(value, &()), ())),
        |_, _| {
            DomainErrorDisposition::Invalid(vec![
                ViolationDraft::new(ViolationCode::new("first")),
                ViolationDraft::new(ViolationCode::new("second")),
            ])
        },
    ))
}

#[test]
fn typed_domain_adapter_passes_metadata_to_error_mapper() {
    #[derive(Debug)]
    struct TypedFailure;
    impl std::fmt::Display for TypedFailure {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("failure")
        }
    }
    impl std::error::Error for TypedFailure {}
    let prepared = prepare_typed_domain_rule::<u32, _, _, _, _, _>(
        &[],
        (),
        |_, value, _| Ok((if *value == 1 { Ok(()) } else { Err(TypedFailure) }, *value)),
        |error, value| {
            if *value == 7 {
                let _ = error;
                DomainErrorDisposition::Invalid(vec![
                    ViolationDraft::new(ViolationCode::new("typed.seven"))
                        .with_param("value", ViolationParam::Unsigned(7)),
                ])
            } else {
                DomainErrorDisposition::Valid
            }
        },
    );
    let input = 7_u32;
    let context = BoundValidationContext::new(&[]);
    assert!(matches!(
        prepared.validate(ValidationValue::Typed(&input), &context).unwrap(),
        PreparedOutcome::Invalid(_)
    ));
}

fn run_text(prepared: &dyn PreparedValidator, input: &str) -> Result<PreparedOutcome, ExecutionError> {
    prepared.validate(ValidationValue::Text(input), &BoundValidationContext::new(&[]))
}
