// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::sync::Arc;

use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidationContext;
use qubit_validator::DependencySpec;
use qubit_validator::ExecutionError;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorSignature;

fn prepare(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    struct Always;
    impl PreparedValidator for Always {
        fn input_type(&self) -> InputType {
            InputType::Text
        }
        fn dependency_specs(&self) -> &'static [DependencySpec] {
            &[]
        }

        fn validate(
            &self,
            _: ValidationValue<'_>,
            _: &BoundValidationContext<'_>,
        ) -> Result<PreparedOutcome, ExecutionError> {
            Ok(PreparedOutcome::Valid)
        }
    }
    Ok(Arc::new(Always))
}

fn prepare_with_error(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Err(BindError::new(BindErrorKind::MissingParameter).with_parameter("minimum"))
}

static SIG_A: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare)];
static D_A: ValidatorDescriptor = ValidatorDescriptor::new(SIG_A);

fn registration(
    id: &'static str,
    file: &'static str,
    descriptor: &'static ValidatorDescriptor,
) -> ValidatorRegistration {
    ValidatorRegistration::new(
        ValidatorId::new(id),
        descriptor,
        RegistrationSource::new("test", "registry", file, 1),
    )
}

#[test]
fn test_duplicate_id_reports_every_source_independent_of_input_order() {
    let one = registration("test.same", "one.rs", &D_A);
    let two = registration("test.same", "two.rs", &D_A);
    let three = registration("test.same", "three.rs", &D_A);
    let error = ValidatorRegistry::from_registrations([three, one, two]).unwrap_err();
    let text = error.to_string();
    assert!(text.contains("one.rs") && text.contains("two.rs") && text.contains("three.rs"));
}

#[test]
fn test_one_id_can_expose_multiple_input_signatures() {
    let signatures: &'static [ValidatorSignature] = Box::leak(Box::new([
        ValidatorSignature::new(InputType::Text, &[], prepare),
        ValidatorSignature::new(InputType::Typed(std::any::TypeId::of::<u32>()), &[], prepare),
    ]));
    let descriptor: &'static ValidatorDescriptor = Box::leak(Box::new(ValidatorDescriptor::new(signatures)));
    let registry = ValidatorRegistry::from_registrations([registration("test.multi", "multi.rs", descriptor)]).unwrap();
    assert_eq!(registry.get("test.multi").unwrap().descriptor().signatures().len(), 2);
}

#[test]
fn test_duplicate_signatures_are_rejected_when_binding() {
    let signatures: &'static [ValidatorSignature] = Box::leak(Box::new([
        ValidatorSignature::new(InputType::Text, &[], prepare),
        ValidatorSignature::new(InputType::Text, &[], prepare),
    ]));
    let descriptor: &'static ValidatorDescriptor = Box::leak(Box::new(ValidatorDescriptor::new(signatures)));
    let error = descriptor
        .bind_for(ValidatorId::new("test.multi"), InputType::Text, &[])
        .unwrap_err();
    assert_eq!(error.kind(), BindErrorKind::AmbiguousSignature);
}

#[test]
fn test_descriptor_rejects_duplicate_input_shapes_and_empty_declarations() {
    static TEXT_DEPS: &[DependencySpec] = &[DependencySpec::new("credential", InputType::Text, false)];
    static DUPLICATE: &[ValidatorSignature] = &[
        ValidatorSignature::new(InputType::Text, &[], prepare),
        ValidatorSignature::new(InputType::Text, TEXT_DEPS, prepare),
    ];
    assert_eq!(
        ValidatorDescriptor::try_new(DUPLICATE).unwrap_err().kind(),
        BindErrorKind::AmbiguousSignature
    );

    static EMPTY: &[ValidatorSignature] = &[];
    assert_eq!(
        ValidatorDescriptor::try_new(EMPTY).unwrap_err().kind(),
        BindErrorKind::InvalidDeclaration
    );
    let descriptor: &'static ValidatorDescriptor = Box::leak(Box::new(ValidatorDescriptor::new(EMPTY)));
    let error =
        ValidatorRegistry::from_registrations([registration("test.empty", "empty.rs", descriptor)]).unwrap_err();
    assert!(error.to_string().contains("invalid descriptor"));
}

#[test]
fn test_binding_rejects_prepared_input_shape_that_disagrees_with_signature() {
    static DECLARED: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::of::<u32>(), &[], prepare)];
    let descriptor = ValidatorDescriptor::new(DECLARED);
    let rule = ValidatorId::new("test.mismatched_shape");

    for result in [
        descriptor.bind(rule, 0, &[]),
        descriptor.bind_for(rule, InputType::of::<u32>(), &[]),
    ] {
        let error = result.expect_err("factory returns a typed prepared validator");
        assert_eq!(error.kind(), BindErrorKind::PreparedSignatureMismatch);
        assert_eq!(error.rule_id(), Some(rule));
    }
}

#[test]
fn test_binding_rejects_prepared_dependency_shape_mismatch() {
    static DECLARED_DEPS: &[DependencySpec] = &[DependencySpec::new("required", InputType::Text, false)];
    static DECLARED: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, DECLARED_DEPS, prepare)];
    let descriptor = ValidatorDescriptor::new(DECLARED);
    let rule = ValidatorId::new("test.mismatched_dependency_shape");
    let error = descriptor
        .bind(rule, 0, &[])
        .expect_err("factory declares no dependencies");
    assert_eq!(error.kind(), BindErrorKind::PreparedSignatureMismatch);
    assert_eq!(error.rule_id(), Some(rule));
}

#[test]
fn test_descriptor_preparation_errors_include_rule_and_keep_details() {
    static FAILING: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare_with_error)];
    static FAILING_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(FAILING);
    let descriptor = ValidatorDescriptor::new(FAILING);
    let rule = ValidatorId::new("test.prepare_failure");

    let error = descriptor.bind(rule, 0, &[]).unwrap_err();
    assert_eq!(error.kind(), BindErrorKind::MissingParameter);
    assert_eq!(error.rule_id(), Some(rule));
    assert_eq!(error.parameter(), Some("minimum"));

    let error = descriptor.bind_for(rule, InputType::Text, &[]).unwrap_err();
    assert_eq!(error.kind(), BindErrorKind::MissingParameter);
    assert_eq!(error.rule_id(), Some(rule));
    assert_eq!(error.parameter(), Some("minimum"));

    let registry = ValidatorRegistry::from_registrations([registration(
        "test.prepare_failure",
        "failing.rs",
        &FAILING_DESCRIPTOR,
    )])
    .unwrap();
    let error = registry.bind("test.prepare_failure", InputType::Text, &[]).unwrap_err();
    assert_eq!(error.kind(), BindErrorKind::MissingParameter);
    assert_eq!(error.rule_id(), Some(rule));
    assert_eq!(error.parameter(), Some("minimum"));
}

#[test]
fn test_descriptor_selection_errors_include_rule() {
    let rule = ValidatorId::new("test.selection");
    let invalid_selection = D_A.bind(rule, 1, &[]).unwrap_err();
    assert_eq!(invalid_selection.kind(), BindErrorKind::InvalidSelection);
    assert_eq!(invalid_selection.rule_id(), Some(rule));

    let unsupported_input = D_A.bind_for(rule, InputType::of::<u32>(), &[]).unwrap_err();
    assert_eq!(unsupported_input.kind(), BindErrorKind::UnsupportedInput);
    assert_eq!(unsupported_input.rule_id(), Some(rule));
}

#[test]
fn test_descriptor_definition_errors_include_rule() {
    static EMPTY: &[ValidatorSignature] = &[];
    let empty = ValidatorDescriptor::new(EMPTY);
    let rule = ValidatorId::new("test.empty_descriptor");
    let error = empty.bind(rule, 0, &[]).unwrap_err();
    assert_eq!(error.kind(), BindErrorKind::InvalidDeclaration);
    assert_eq!(error.rule_id(), Some(rule));

    static DUPLICATE: &[ValidatorSignature] = &[
        ValidatorSignature::new(InputType::Text, &[], prepare),
        ValidatorSignature::new(InputType::Text, &[], prepare),
    ];
    let error = ValidatorDescriptor::new(DUPLICATE)
        .bind_for(rule, InputType::Text, &[])
        .unwrap_err();
    assert_eq!(error.kind(), BindErrorKind::AmbiguousSignature);
    assert_eq!(error.rule_id(), Some(rule));
}
