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
use qubit_validator::NamedValidationDependency;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorSignature;

struct ValidAdapter;

impl PreparedValidator for ValidAdapter {
    fn validate(
        &self,
        _: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        Ok(PreparedOutcome::Valid)
    }
}

fn prepare_valid(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(ValidAdapter))
}

static DEPENDENCIES: &[DependencySpec] = &[
    DependencySpec::new("minimum", InputType::of::<u64>(), false),
    DependencySpec::new("maximum", InputType::of::<u64>(), false),
];
static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, DEPENDENCIES, prepare_valid)];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
const RULE_ID: ValidatorId = ValidatorId::new("test.dependency_contract");

struct OrderedTextAdapter;

impl PreparedValidator for OrderedTextAdapter {
    fn validate(
        &self,
        _: ValidationValue<'_>,
        context: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        assert_eq!(context.text(0)?, "first value");
        assert_eq!(context.text(1)?, "second value");
        Ok(PreparedOutcome::valid())
    }
}

fn prepare_ordered_text(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(OrderedTextAdapter))
}

static TEXT_DEPENDENCIES: &[DependencySpec] = &[
    DependencySpec::new("first", InputType::Text, false),
    DependencySpec::new("second", InputType::Text, false),
];
static TEXT_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    TEXT_DEPENDENCIES,
    prepare_ordered_text,
)];
static TEXT_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(TEXT_SIGNATURES);

#[test]
fn test_bind_takes_dependency_specs_from_selected_signature() {
    let bound = DESCRIPTOR.bind(RULE_ID, 0, &[]).expect("signature binds");
    assert_eq!(bound.dependency_specs(), DEPENDENCIES);
}

#[test]
fn test_validate_reports_missing_dependency_name_rule_and_path() {
    let bound = DESCRIPTOR
        .bind(RULE_ID, 0, &[])
        .expect("dependency slots in signature order must bind");
    let maximum = 10_u64;
    let values = [ValidationValue::Missing, ValidationValue::Typed(&maximum)];
    let minimum_path = ValidationPath::root().with_field("range").with_field("minimum");
    let maximum_path = ValidationPath::root().with_field("range").with_field("maximum");
    let paths = [minimum_path.clone(), maximum_path];
    let context = BoundValidationContext::new_with_paths(&values, &paths)
        .expect("dependency values and paths have matching lengths");

    let error = bound
        .validate(ValidationValue::Text("value"), &context)
        .expect_err("a required missing dependency must fail before adapter execution");

    assert_eq!(error.kind(), ExecutionErrorKind::MissingRequiredDependencyValue);
    assert_eq!(error.rule_id(), Some(RULE_ID));
    assert_eq!(error.dependency(), Some("minimum"));
    assert_eq!(error.path(), &minimum_path);
}

#[test]
fn test_validate_reports_wrong_dependency_type_name_rule_and_path() {
    let bound = DESCRIPTOR
        .bind(RULE_ID, 0, &[])
        .expect("dependency slots in signature order must bind");
    let minimum = 1_u64;
    let values = [ValidationValue::Typed(&minimum), ValidationValue::Text("ten")];
    let minimum_path = ValidationPath::root().with_field("range").with_field("minimum");
    let maximum_path = ValidationPath::root().with_field("range").with_field("maximum");
    let paths = [minimum_path, maximum_path.clone()];
    let context = BoundValidationContext::new_with_paths(&values, &paths)
        .expect("dependency values and paths have matching lengths");

    let error = bound
        .validate(ValidationValue::Text("value"), &context)
        .expect_err("a dependency with the wrong type must fail before adapter execution");

    assert_eq!(error.kind(), ExecutionErrorKind::DependencyTypeMismatch);
    assert_eq!(error.rule_id(), Some(RULE_ID));
    assert_eq!(error.dependency(), Some("maximum"));
    assert_eq!(error.path(), &maximum_path);
}

#[test]
fn test_context_rejects_mismatched_value_and_path_counts_at_root() {
    let minimum = 1_u64;
    let values = [ValidationValue::Typed(&minimum)];

    let error =
        BoundValidationContext::new_with_paths(&values, &[]).expect_err("dependency value and path counts must match");

    assert_eq!(error.kind(), ExecutionErrorKind::AdapterContractViolation);
    assert_eq!(error.dependency(), None);
    assert_eq!(error.path(), &ValidationPath::root());
}

#[test]
fn test_validate_named_reorders_same_type_dependencies_and_paths() {
    let bound = TEXT_DESCRIPTOR.bind(RULE_ID, 0, &[]).expect("signature binds");
    let first_path = ValidationPath::root().with_field("first_value");
    let second_path = ValidationPath::root().with_field("second_value");
    let dependencies = [
        NamedValidationDependency::new("second", ValidationValue::Text("second value")).with_path(&second_path),
        NamedValidationDependency::new("first", ValidationValue::Text("first value")).with_path(&first_path),
    ];

    assert_eq!(
        bound
            .validate_named(ValidationValue::Text("target"), &dependencies)
            .expect("named dependencies are reordered by signature"),
        qubit_validator::ValidationOutcome::Valid,
    );
}

#[test]
fn test_validate_named_rejects_duplicate_unknown_and_missing_names() {
    let bound = TEXT_DESCRIPTOR.bind(RULE_ID, 0, &[]).expect("signature binds");
    let first = NamedValidationDependency::new("first", ValidationValue::Text("first value"));
    let second = NamedValidationDependency::new("second", ValidationValue::Text("second value"));

    let duplicate = [first, first];
    let error = bound
        .validate_named(ValidationValue::Text("target"), &duplicate)
        .expect_err("duplicate names are rejected");
    assert_eq!(error.kind(), ExecutionErrorKind::DuplicateDependencyBinding);
    assert_eq!(error.rule_id(), Some(RULE_ID));
    assert_eq!(error.dependency(), Some("first"));

    let unknown = [
        first,
        second,
        NamedValidationDependency::new("other", ValidationValue::Text("hidden")),
    ];
    let error = bound
        .validate_named(ValidationValue::Text("target"), &unknown)
        .expect_err("unknown names are rejected");
    assert_eq!(error.kind(), ExecutionErrorKind::UnknownDependencyBinding);
    assert_eq!(error.dependency(), Some("other"));
    assert!(!format!("{error:?}").contains("hidden"));

    let missing = [first];
    let error = bound
        .validate_named(ValidationValue::Text("target"), &missing)
        .expect_err("every declared slot must be supplied");
    assert_eq!(error.kind(), ExecutionErrorKind::MissingDependencyBinding);
    assert_eq!(error.dependency(), Some("second"));
}

#[test]
fn test_validate_named_reports_dependency_shape_with_rule_and_path() {
    let bound = TEXT_DESCRIPTOR.bind(RULE_ID, 0, &[]).expect("signature binds");
    let wrong = 7_u64;
    let first_path = ValidationPath::root().with_field("first_value");
    let dependencies = [
        NamedValidationDependency::new("first", ValidationValue::Typed(&wrong)).with_path(&first_path),
        NamedValidationDependency::new("second", ValidationValue::Text("second value")),
    ];

    let error = bound
        .validate_named(ValidationValue::Text("target"), &dependencies)
        .expect_err("a dependency with the wrong type is rejected");
    assert_eq!(error.kind(), ExecutionErrorKind::DependencyTypeMismatch);
    assert_eq!(error.rule_id(), Some(RULE_ID));
    assert_eq!(error.dependency(), Some("first"));
    assert_eq!(error.path(), &first_path);
}
