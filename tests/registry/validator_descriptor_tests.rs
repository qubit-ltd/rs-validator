// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::sync::Arc;

use qubit_validator::ArgumentReader;
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
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorSignature;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;
use qubit_validator::prepare_typed_with_context;

fn prepare(_reader: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
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

fn prepare_with_error(_reader: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Err(BindError::new(BindErrorKind::MissingParameter).with_parameter("minimum"))
}

fn prepare_ignoring_reader(_reader: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
    prepare(_reader)
}

fn prepare_requiring_maximum(reader: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let _maximum = reader.required_u32("maximum")?;
    prepare_ignoring_reader(reader)
}

fn prepare_multi_u32(_reader: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(prepare_typed_with_context::<u32, _>(&[], |_value, _context| {
        Ok(PreparedOutcome::Invalid(vec![ViolationDraft::new(ViolationCode::new(
            "test.typed_selected",
        ))]))
    }))
}

static SIG_A: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare)];
static D_A: ValidatorDescriptor = ValidatorDescriptor::new(SIG_A);

#[test]
fn test_bind_rejects_duplicate_parameter_even_when_factory_ignores_reader() {
    static SIGNATURES: &[ValidatorSignature] =
        &[ValidatorSignature::new(InputType::Text, &[], prepare_ignoring_reader)];
    let descriptor = ValidatorDescriptor::new(SIGNATURES);
    let rule = ValidatorId::new("test.duplicate_parameter");
    let params = [
        NamedValidationArgument::new("extra", ValidationArgument::Unsigned(1)),
        NamedValidationArgument::new("extra", ValidationArgument::Unsigned(2)),
    ];

    let error = descriptor
        .bind(rule, 0, &params)
        .expect_err("duplicate parameter must be rejected");
    assert_eq!(error.kind(), BindErrorKind::DuplicateParameter);
    assert_eq!(error.rule_id(), Some(rule));
}

#[test]
fn test_bind_rejects_unconsumed_parameter() {
    static SIGNATURES: &[ValidatorSignature] =
        &[ValidatorSignature::new(InputType::Text, &[], prepare_ignoring_reader)];
    let descriptor = ValidatorDescriptor::new(SIGNATURES);
    let rule = ValidatorId::new("test.unconsumed_parameter");
    let params = [NamedValidationArgument::new("extra", ValidationArgument::Unsigned(1))];

    let error = descriptor
        .bind(rule, 0, &params)
        .expect_err("unconsumed parameter must be rejected");
    assert_eq!(error.kind(), BindErrorKind::UnknownParameter);
    assert_eq!(error.rule_id(), Some(rule));
}

#[test]
fn test_bind_for_rejects_unconsumed_parameter() {
    static SIGNATURES: &[ValidatorSignature] =
        &[ValidatorSignature::new(InputType::Text, &[], prepare_ignoring_reader)];
    let descriptor = ValidatorDescriptor::new(SIGNATURES);
    let rule = ValidatorId::new("test.unconsumed_parameter_for");
    let params = [NamedValidationArgument::new("extra", ValidationArgument::Unsigned(1))];

    let error = descriptor
        .bind_for(rule, InputType::Text, &params)
        .expect_err("unconsumed parameter must be rejected");
    assert_eq!(error.kind(), BindErrorKind::UnknownParameter);
    assert_eq!(error.rule_id(), Some(rule));
}

#[test]
fn test_bind_reports_missing_required_parameter() {
    static SIGNATURES: &[ValidatorSignature] =
        &[ValidatorSignature::new(InputType::Text, &[], prepare_requiring_maximum)];
    let descriptor = ValidatorDescriptor::new(SIGNATURES);
    let rule = ValidatorId::new("test.missing_maximum");

    let error = descriptor
        .bind(rule, 0, &[])
        .expect_err("missing required parameter must be reported");
    assert_eq!(error.kind(), BindErrorKind::MissingParameter);
    assert_eq!(error.rule_id(), Some(rule));
}

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
    let error =
        ValidatorRegistry::from_registrations([three, one, two]).expect_err("duplicate validator IDs must be rejected");
    let text = error.to_string();
    assert!(text.contains("one.rs") && text.contains("two.rs") && text.contains("three.rs"));
}

#[test]
fn test_one_id_can_expose_multiple_input_signatures() {
    for reversed in [false, true] {
        let mut declarations = [
            ValidatorSignature::new(InputType::Text, &[], prepare),
            ValidatorSignature::new(InputType::of::<u32>(), &[], prepare_multi_u32),
        ];
        if reversed {
            declarations.reverse();
        }
        let signatures: &'static [ValidatorSignature] = Box::leak(Box::new(declarations));
        let descriptor: &'static ValidatorDescriptor = Box::leak(Box::new(ValidatorDescriptor::new(signatures)));
        let registry = ValidatorRegistry::from_registrations([registration("test.multi", "multi.rs", descriptor)])
            .expect("both input signatures have matching prepared implementations");
        let registered = registry.get("test.multi").expect("multi rule is registered");
        assert_eq!(registered.descriptor().signatures().len(), 2);
        let text = registry
            .bind("test.multi", InputType::Text, &[])
            .expect("text signature binds in either declaration order");
        let typed = registry
            .bind("test.multi", InputType::of::<u32>(), &[])
            .expect("u32 signature binds in either declaration order");
        assert_eq!(
            text.validate(ValidationValue::Text("ready"), &BoundValidationContext::new(&[]))
                .expect("text rule executes"),
            ValidationOutcome::Valid,
            "reversed={reversed}",
        );
        let value = 7_u32;
        let outcome = typed
            .validate(ValidationValue::Typed(&value), &BoundValidationContext::new(&[]))
            .expect("typed domain rejection is an outcome");
        let ValidationOutcome::Invalid(violations) = outcome else {
            panic!("u32 must execute its own rejecting factory; reversed={reversed}");
        };
        assert_eq!(violations.len(), 1, "reversed={reversed}");
        assert_eq!(violations[0].rule_id(), ValidatorId::new("test.multi"));
        assert_eq!(violations[0].code(), ViolationCode::new("test.typed_selected"));
    }
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
        .expect_err("duplicate signatures must be rejected");
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
        ValidatorDescriptor::try_new(DUPLICATE)
            .expect_err("duplicate input shapes must be rejected")
            .kind(),
        BindErrorKind::AmbiguousSignature
    );

    static EMPTY: &[ValidatorSignature] = &[];
    assert_eq!(
        ValidatorDescriptor::try_new(EMPTY)
            .expect_err("empty declarations must be rejected")
            .kind(),
        BindErrorKind::InvalidDeclaration
    );
    let descriptor: &'static ValidatorDescriptor = Box::leak(Box::new(ValidatorDescriptor::new(EMPTY)));
    let error = ValidatorRegistry::from_registrations([registration("test.empty", "empty.rs", descriptor)])
        .expect_err("a registry must reject an invalid descriptor");
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

    let error = descriptor
        .bind(rule, 0, &[])
        .expect_err("the preparation error must be returned");
    assert_eq!(error.kind(), BindErrorKind::MissingParameter);
    assert_eq!(error.rule_id(), Some(rule));
    assert_eq!(error.parameter(), Some("minimum"));

    let error = descriptor
        .bind_for(rule, InputType::Text, &[])
        .expect_err("the preparation error must be returned");
    assert_eq!(error.kind(), BindErrorKind::MissingParameter);
    assert_eq!(error.rule_id(), Some(rule));
    assert_eq!(error.parameter(), Some("minimum"));

    let registry = ValidatorRegistry::from_registrations([registration(
        "test.prepare_failure",
        "failing.rs",
        &FAILING_DESCRIPTOR,
    )])
    .expect("the failing descriptor is structurally valid");
    let error = registry
        .bind("test.prepare_failure", InputType::Text, &[])
        .expect_err("the preparation error must be returned");
    assert_eq!(error.kind(), BindErrorKind::MissingParameter);
    assert_eq!(error.rule_id(), Some(rule));
    assert_eq!(error.parameter(), Some("minimum"));
}

#[test]
fn test_descriptor_selection_errors_include_rule() {
    let rule = ValidatorId::new("test.selection");
    let invalid_selection = D_A
        .bind(rule, 1, &[])
        .expect_err("an out-of-range signature selection must be rejected");
    assert_eq!(invalid_selection.kind(), BindErrorKind::InvalidSelection);
    assert_eq!(invalid_selection.rule_id(), Some(rule));

    let unsupported_input = D_A
        .bind_for(rule, InputType::of::<u32>(), &[])
        .expect_err("an unsupported input type must be rejected");
    assert_eq!(unsupported_input.kind(), BindErrorKind::UnsupportedInput);
    assert_eq!(unsupported_input.rule_id(), Some(rule));
}

#[test]
fn test_descriptor_definition_errors_include_rule() {
    static EMPTY: &[ValidatorSignature] = &[];
    let empty = ValidatorDescriptor::new(EMPTY);
    let rule = ValidatorId::new("test.empty_descriptor");
    let error = empty
        .bind(rule, 0, &[])
        .expect_err("an empty descriptor must be rejected");
    assert_eq!(error.kind(), BindErrorKind::InvalidDeclaration);
    assert_eq!(error.rule_id(), Some(rule));

    static DUPLICATE: &[ValidatorSignature] = &[
        ValidatorSignature::new(InputType::Text, &[], prepare),
        ValidatorSignature::new(InputType::Text, &[], prepare),
    ];
    let error = ValidatorDescriptor::new(DUPLICATE)
        .bind_for(rule, InputType::Text, &[])
        .expect_err("duplicate signatures must be rejected");
    assert_eq!(error.kind(), BindErrorKind::AmbiguousSignature);
    assert_eq!(error.rule_id(), Some(rule));
}
