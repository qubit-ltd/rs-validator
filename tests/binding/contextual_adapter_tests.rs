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
use qubit_validator::prepare_contextual_text_validator;
use qubit_validator::prepare_contextual_typed_validator;
use qubit_validator::prepare_text_with_context;
use qubit_validator::prepare_typed_with_context;

#[derive(Debug, thiserror::Error)]
#[error("text does not match the dependency pair")]
struct DependencyPairMismatch;

struct MatchesDependencyPair;

impl<'a> Validator<str, BoundValidationContext<'a>> for MatchesDependencyPair {
    type Error = DependencyPairMismatch;

    fn validate(&self, value: &str, context: &BoundValidationContext<'a>) -> Result<(), Self::Error> {
        let first = context.text(0).map_err(|_| DependencyPairMismatch)?;
        let second = context.text(1).map_err(|_| DependencyPairMismatch)?;
        if value == format!("{first}:{second}") {
            Ok(())
        } else {
            Err(DependencyPairMismatch)
        }
    }
}

static TEXT_DEPENDENCIES: &[DependencySpec] = &[
    DependencySpec::new("first", InputType::Text, false),
    DependencySpec::new("second", InputType::Text, false),
];

fn prepare_dependency_pair(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(prepare_contextual_text_validator(
        TEXT_DEPENDENCIES,
        MatchesDependencyPair,
        |_| ViolationDraft::new(ViolationCode::new("text.dependency_mismatch")),
    ))
}

static TEXT_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    TEXT_DEPENDENCIES,
    prepare_dependency_pair,
)];
static TEXT_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(TEXT_SIGNATURES);

#[derive(Debug, thiserror::Error)]
#[error("value is below its optional minimum")]
struct BelowMinimum;

struct OptionalMinimum;

impl<'a> Validator<u32, BoundValidationContext<'a>> for OptionalMinimum {
    type Error = BelowMinimum;

    fn validate(&self, value: &u32, context: &BoundValidationContext<'a>) -> Result<(), Self::Error> {
        let Some(minimum) = context.optional_typed::<u32>(0).map_err(|_| BelowMinimum)? else {
            return Ok(());
        };
        if value >= minimum { Ok(()) } else { Err(BelowMinimum) }
    }
}

static OPTIONAL_DEPENDENCIES: &[DependencySpec] = &[DependencySpec::new("minimum", InputType::of::<u32>(), true)];

fn prepare_optional_minimum(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(prepare_contextual_typed_validator::<u32, _, _, _>(
        OPTIONAL_DEPENDENCIES,
        OptionalMinimum,
        |_| ViolationDraft::new(ViolationCode::new("number.below_minimum")),
    ))
}

static TYPED_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::of::<u32>(),
    OPTIONAL_DEPENDENCIES,
    prepare_optional_minimum,
)];
static TYPED_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(TYPED_SIGNATURES);

#[test]
fn test_contextual_text_adapter_reads_dependency_slots_in_order() {
    let bound = TEXT_DESCRIPTOR
        .bind_for(ValidatorId::new("test.matches_pair"), InputType::Text, &[])
        .expect("declared dependency pair binds");
    let values = [ValidationValue::Text("alpha"), ValidationValue::Text("beta")];
    let context = BoundValidationContext::new(&values);

    assert_eq!(
        bound
            .validate(ValidationValue::Text("alpha:beta"), &context)
            .expect("matching dependencies pass"),
        ValidationOutcome::Valid,
    );
    let outcome = bound
        .validate(ValidationValue::Text("beta:alpha"), &context)
        .expect("a domain mismatch is a validation outcome");
    assert!(matches!(&outcome, ValidationOutcome::Invalid(violations)
        if violations.len() == 1
            && violations[0].code().as_str() == "text.dependency_mismatch"));
    assert!(!format!("{outcome:?}").contains("alpha"));
}

#[test]
fn test_contextual_typed_adapter_accepts_present_and_missing_optional_slot() {
    let bound = TYPED_DESCRIPTOR
        .bind_for(ValidatorId::new("test.optional_minimum"), InputType::of::<u32>(), &[])
        .expect("optional minimum dependency binds");
    let value = 5_u32;
    let minimum = 3_u32;
    let present = [ValidationValue::Typed(&minimum)];
    let missing = [ValidationValue::Missing];

    assert_eq!(
        bound
            .validate(ValidationValue::Typed(&value), &BoundValidationContext::new(&present),)
            .expect("value meets present minimum"),
        ValidationOutcome::Valid,
    );
    assert_eq!(
        bound
            .validate(ValidationValue::Typed(&value), &BoundValidationContext::new(&missing),)
            .expect("missing optional minimum is accepted"),
        ValidationOutcome::Valid,
    );
}

#[test]
fn test_contextual_adapters_reject_target_shape_before_invocation() {
    let text = prepare_contextual_text_validator(TEXT_DEPENDENCIES, MatchesDependencyPair, |_| {
        ViolationDraft::new(ViolationCode::new("text.dependency_mismatch"))
    });
    let error = text
        .validate(ValidationValue::Typed(&1_u32), &BoundValidationContext::new(&[]))
        .expect_err("text adapter must reject a typed target");
    assert_eq!(error.kind(), ExecutionErrorKind::InputTypeMismatch);

    let typed = prepare_contextual_typed_validator::<u32, _, _, _>(OPTIONAL_DEPENDENCIES, OptionalMinimum, |_| {
        ViolationDraft::new(ViolationCode::new("number.below_minimum"))
    });
    let error = typed
        .validate(ValidationValue::Text("5"), &BoundValidationContext::new(&[]))
        .expect_err("typed adapter must reject a text target");
    assert_eq!(error.kind(), ExecutionErrorKind::InputTypeMismatch);
}

#[test]
fn test_bound_validator_reports_dependency_slot_path_before_contextual_adapter() {
    let bound = TEXT_DESCRIPTOR
        .bind_for(ValidatorId::new("test.matches_pair"), InputType::Text, &[])
        .expect("declared dependency pair binds");
    let values = [ValidationValue::Missing, ValidationValue::Text("beta")];
    let first_path = ValidationPath::root().with_field("profile").with_field("first");
    let paths = [
        first_path.clone(),
        ValidationPath::root().with_field("profile").with_field("second"),
    ];
    let context =
        BoundValidationContext::new_with_paths(&values, &paths).expect("dependency paths correspond to the slots");

    let error = bound
        .validate(ValidationValue::Text("alpha:beta"), &context)
        .expect_err("missing required dependency must fail before the adapter");
    assert_eq!(error.kind(), ExecutionErrorKind::MissingRequiredDependencyValue);
    assert_eq!(error.dependency(), Some("first"));
    assert_eq!(error.path(), &first_path);
}

#[test]
fn test_text_context_closure_preserves_all_prepared_outcomes_and_execution_errors() {
    let text = prepare_text_with_context(&[], |value, _| {
        if value == "accept" {
            Ok(PreparedOutcome::Valid)
        } else if value == "reject twice" {
            PreparedOutcome::invalid(vec![
                ViolationDraft::new(ViolationCode::new("text.first")),
                ViolationDraft::new(ViolationCode::new("text.second")),
            ])
            .map_err(|_| ExecutionError::new(ExecutionErrorKind::AdapterContractViolation))
        } else {
            Err(ExecutionError::new(ExecutionErrorKind::ExternalFailure))
        }
    });

    assert_eq!(
        text.validate(ValidationValue::Text("accept"), &BoundValidationContext::new(&[]))
            .expect("closure can return a valid prepared outcome"),
        PreparedOutcome::Valid,
    );
    assert!(matches!(
        text.validate(ValidationValue::Text("reject twice"), &BoundValidationContext::new(&[]))
            .expect("closure can return multiple violation drafts"),
        PreparedOutcome::Invalid(drafts) if drafts.len() == 2
    ));
    let error = text
        .validate(
            ValidationValue::Text("external failure"),
            &BoundValidationContext::new(&[]),
        )
        .expect_err("execution errors must remain execution errors");
    assert_eq!(error.kind(), ExecutionErrorKind::ExternalFailure);
}

#[test]
fn test_typed_context_closure_rejects_wrong_target_type_before_invocation() {
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    let calls = Arc::new(AtomicUsize::new(0));
    let prepared = prepare_typed_with_context::<u32, _>(&[], {
        let calls = Arc::clone(&calls);
        move |value, _| {
            calls.fetch_add(1, Ordering::SeqCst);
            if *value == 0 {
                Err(ExecutionError::new(ExecutionErrorKind::ExternalFailure))
            } else {
                Ok(PreparedOutcome::Valid)
            }
        }
    });

    let wrong_type = prepared
        .validate(ValidationValue::Text("secret"), &BoundValidationContext::new(&[]))
        .expect_err("typed closure must reject text before invocation");
    assert_eq!(wrong_type.kind(), ExecutionErrorKind::InputTypeMismatch);
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    let zero = 0_u32;
    let error = prepared
        .validate(ValidationValue::Typed(&zero), &BoundValidationContext::new(&[]))
        .expect_err("closure execution failures must propagate");
    assert_eq!(error.kind(), ExecutionErrorKind::ExternalFailure);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

static FALLIBLE_DEPENDENCIES: &[DependencySpec] = &[DependencySpec::new("expected", InputType::Text, false)];

fn prepare_fallible_dependency(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(prepare_text_with_context(FALLIBLE_DEPENDENCIES, |_, context| {
        context.text(0)?;
        Err(ExecutionError::new(ExecutionErrorKind::ExternalFailure))
    }))
}

static FALLIBLE_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    FALLIBLE_DEPENDENCIES,
    prepare_fallible_dependency,
)];
static FALLIBLE_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(FALLIBLE_SIGNATURES);

#[test]
fn test_bound_validator_attaches_rule_to_context_closure_execution_error() {
    let rule_id = ValidatorId::new("test.context_failure");
    let bound = FALLIBLE_DESCRIPTOR
        .bind_for(rule_id, InputType::Text, &[])
        .expect("fallible contextual validator binds");
    let values = [ValidationValue::Text("expected")];
    let paths = [ValidationPath::root().with_field("profile").with_field("expected")];
    let context = BoundValidationContext::new_with_paths(&values, &paths).expect("one path per dependency");

    let error = bound
        .validate(ValidationValue::Text("candidate"), &context)
        .expect_err("closure execution failure must not become a validation violation");
    assert_eq!(error.kind(), ExecutionErrorKind::ExternalFailure);
    assert_eq!(error.rule_id(), Some(rule_id));
}
