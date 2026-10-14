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
use qubit_validator::BoundValidationContext;
use qubit_validator::BoundValidator;
use qubit_validator::DependencySpec;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::InputType;
use qubit_validator::NamedValidationDependency;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorSignature;

struct ValidAdapter;

impl PreparedValidator for ValidAdapter {
    fn input_type(&self) -> InputType {
        InputType::Text
    }
    fn dependency_specs(&self) -> &'static [DependencySpec] {
        DEPENDENCIES
    }

    fn validate(
        &self,
        _: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        Ok(PreparedOutcome::Valid)
    }
}

fn prepare_valid(_reader: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
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
    fn input_type(&self) -> InputType {
        InputType::Text
    }
    fn dependency_specs(&self) -> &'static [DependencySpec] {
        TEXT_DEPENDENCIES
    }

    fn validate(
        &self,
        _: ValidationValue<'_>,
        context: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        assert_eq!(context.text(0)?, "first value");
        assert_eq!(context.text(1)?, "second value");
        assert_eq!(
            context.dependency_path(0)?,
            &ValidationPath::root().with_field("first_value")
        );
        assert_eq!(
            context.dependency_path(1)?,
            &ValidationPath::root().with_field("second_value")
        );
        Ok(PreparedOutcome::valid())
    }
}

fn prepare_ordered_text(_reader: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(OrderedTextAdapter))
}

struct RootFirstTextAdapter;

impl PreparedValidator for RootFirstTextAdapter {
    fn input_type(&self) -> InputType {
        InputType::Text
    }

    fn dependency_specs(&self) -> &'static [DependencySpec] {
        TEXT_DEPENDENCIES
    }

    fn validate(
        &self,
        _: ValidationValue<'_>,
        context: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        assert_eq!(context.text(0)?, "first value");
        assert_eq!(context.text(1)?, "second value");
        assert_eq!(context.dependency_path(0)?, &ValidationPath::root());
        assert_eq!(
            context.dependency_path(1)?,
            &ValidationPath::root().with_field("second_value")
        );
        Ok(PreparedOutcome::valid())
    }
}

fn prepare_root_first_text(_reader: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(RootFirstTextAdapter))
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
static ROOT_FIRST_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    TEXT_DEPENDENCIES,
    prepare_root_first_text,
)];
static ROOT_FIRST_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(ROOT_FIRST_SIGNATURES);

struct NineSlotAdapter;

impl PreparedValidator for NineSlotAdapter {
    fn input_type(&self) -> InputType {
        InputType::Text
    }

    fn dependency_specs(&self) -> &'static [DependencySpec] {
        NINE_DEPENDENCIES
    }

    fn validate(
        &self,
        _: ValidationValue<'_>,
        context: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        for index in 0..9 {
            assert_eq!(*context.typed::<u64>(index)?, index as u64);
            assert_eq!(context.dependency_path(index)?, &ValidationPath::root());
        }
        Ok(PreparedOutcome::valid())
    }
}

fn prepare_nine_slot(_reader: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(NineSlotAdapter))
}

static NINE_DEPENDENCIES: &[DependencySpec] = &[
    DependencySpec::new("d0", InputType::of::<u64>(), false),
    DependencySpec::new("d1", InputType::of::<u64>(), false),
    DependencySpec::new("d2", InputType::of::<u64>(), false),
    DependencySpec::new("d3", InputType::of::<u64>(), false),
    DependencySpec::new("d4", InputType::of::<u64>(), false),
    DependencySpec::new("d5", InputType::of::<u64>(), false),
    DependencySpec::new("d6", InputType::of::<u64>(), false),
    DependencySpec::new("d7", InputType::of::<u64>(), false),
    DependencySpec::new("d8", InputType::of::<u64>(), false),
];
static NINE_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    NINE_DEPENDENCIES,
    prepare_nine_slot,
)];
static NINE_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(NINE_SIGNATURES);

struct NinePathAdapter;

impl PreparedValidator for NinePathAdapter {
    fn input_type(&self) -> InputType {
        InputType::Text
    }

    fn dependency_specs(&self) -> &'static [DependencySpec] {
        NINE_DEPENDENCIES
    }

    fn validate(
        &self,
        _: ValidationValue<'_>,
        context: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        for index in 0..9 {
            assert_eq!(*context.typed::<u64>(index)?, index as u64);
            let expected_path = match index {
                0 => nine_path_zero(),
                8 => nine_path_eight(),
                _ => ValidationPath::root(),
            };
            assert_eq!(context.dependency_path(index)?, &expected_path);
        }
        Ok(PreparedOutcome::valid())
    }
}

fn prepare_nine_path(_reader: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(NinePathAdapter))
}

fn nine_path_zero() -> ValidationPath {
    ValidationPath::root().with_field("nine_path_zero")
}

fn nine_path_eight() -> ValidationPath {
    ValidationPath::root().with_field("nine_path_eight")
}

static NINE_PATH_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    NINE_DEPENDENCIES,
    prepare_nine_path,
)];
static NINE_PATH_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(NINE_PATH_SIGNATURES);

struct NoDependencyAdapter;

impl PreparedValidator for NoDependencyAdapter {
    fn input_type(&self) -> InputType {
        InputType::of::<String>()
    }

    fn dependency_specs(&self) -> &'static [DependencySpec] {
        &[]
    }

    fn validate(
        &self,
        _: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        Ok(PreparedOutcome::valid())
    }
}

struct SingleTextAdapter;

impl PreparedValidator for SingleTextAdapter {
    fn input_type(&self) -> InputType {
        InputType::Text
    }

    fn dependency_specs(&self) -> &'static [DependencySpec] {
        SINGLE_DEPENDENCIES
    }

    fn validate(
        &self,
        _: ValidationValue<'_>,
        context: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        assert_eq!(context.text(0)?, "dependency value");
        Ok(PreparedOutcome::valid())
    }
}

fn prepare_single_text(_reader: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(SingleTextAdapter))
}

static SINGLE_DEPENDENCIES: &[DependencySpec] = &[DependencySpec::new("item", InputType::Text, false)];
static SINGLE_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    SINGLE_DEPENDENCIES,
    prepare_single_text,
)];
static SINGLE_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SINGLE_SIGNATURES);

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
        ValidationOutcome::Valid,
    );
}

#[test]
fn test_validate_named_defaults_missing_dependency_path_to_root_after_reordering() {
    let bound = ROOT_FIRST_DESCRIPTOR.bind(RULE_ID, 0, &[]).expect("signature binds");
    let second_path = ValidationPath::root().with_field("second_value");
    let dependencies = [
        NamedValidationDependency::new("second", ValidationValue::Text("second value")).with_path(&second_path),
        NamedValidationDependency::new("first", ValidationValue::Text("first value")),
    ];

    assert_eq!(
        bound
            .validate_named(ValidationValue::Text("target"), &dependencies)
            .expect("named dependencies are reordered and missing paths default to root"),
        ValidationOutcome::Valid,
    );
}

#[test]
fn test_validate_named_reorders_nine_dependencies_without_paths() {
    let bound = NINE_DESCRIPTOR.bind(RULE_ID, 0, &[]).expect("signature binds");
    let numbers: [u64; 9] = std::array::from_fn(|index| index as u64);
    let dependencies: [NamedValidationDependency<'_>; 9] = std::array::from_fn(|index| {
        let slot = 8 - index;
        NamedValidationDependency::new(NINE_DEPENDENCIES[slot].name(), ValidationValue::Typed(&numbers[slot]))
    });

    assert_eq!(
        bound
            .validate_named(ValidationValue::Text("target"), &dependencies)
            .expect("nine named dependencies are reordered by signature"),
        ValidationOutcome::Valid,
    );
}

#[test]
fn test_validate_named_nine_dependencies_reorders_mixed_paths() {
    let bound = NINE_PATH_DESCRIPTOR.bind(RULE_ID, 0, &[]).expect("signature binds");
    let numbers: [u64; 9] = std::array::from_fn(|index| index as u64);
    let path_zero = nine_path_zero();
    let path_eight = nine_path_eight();
    let dependencies: [NamedValidationDependency<'_>; 9] = std::array::from_fn(|index| {
        let slot = 8 - index;
        let dependency =
            NamedValidationDependency::new(NINE_DEPENDENCIES[slot].name(), ValidationValue::Typed(&numbers[slot]));
        match slot {
            0 => dependency.with_path(&path_zero),
            8 => dependency.with_path(&path_eight),
            _ => dependency,
        }
    });

    assert_eq!(
        bound
            .validate_named(ValidationValue::Text("target"), &dependencies)
            .expect("nine named dependencies are reordered with mixed paths"),
        ValidationOutcome::Valid,
    );
}

#[test]
fn test_validate_named_nine_dependencies_preserves_type_error_path() {
    let bound = NINE_PATH_DESCRIPTOR.bind(RULE_ID, 0, &[]).expect("signature binds");
    let numbers: [u64; 9] = std::array::from_fn(|index| index as u64);
    let error_path = nine_path_eight();
    let dependencies: [NamedValidationDependency<'_>; 9] = std::array::from_fn(|index| {
        let slot = 8 - index;
        if slot == 8 {
            NamedValidationDependency::new("d8", ValidationValue::Text("wrong")).with_path(&error_path)
        } else {
            NamedValidationDependency::new(NINE_DEPENDENCIES[slot].name(), ValidationValue::Typed(&numbers[slot]))
        }
    });

    let error = bound
        .validate_named(ValidationValue::Text("target"), &dependencies)
        .expect_err("the type mismatch keeps the supplied slot path");
    assert_eq!(error.kind(), ExecutionErrorKind::DependencyTypeMismatch);
    assert_eq!(error.rule_id(), Some(RULE_ID));
    assert_eq!(error.dependency(), Some("d8"));
    assert_eq!(error.path(), &error_path);
}

#[test]
fn test_validate_named_duplicate_takes_priority_over_missing_slot() {
    let bound = TEXT_DESCRIPTOR.bind(RULE_ID, 0, &[]).expect("signature binds");
    let first = NamedValidationDependency::new("first", ValidationValue::Text("first value"));

    let error = bound
        .validate_named(ValidationValue::Text("target"), &[first, first])
        .expect_err("duplicate input is reported before a missing signature slot");
    assert_eq!(error.kind(), ExecutionErrorKind::DuplicateDependencyBinding);
    assert_eq!(error.rule_id(), Some(RULE_ID));
    assert_eq!(error.dependency(), Some("first"));
    assert_eq!(error.path(), &ValidationPath::root());
    assert!(!format!("{error:?}").contains("first value"));
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

#[test]
fn test_validate_named_zero_dependencies_matches_ordered_validation() {
    let bound = BoundValidator::try_from_prepared::<String>(RULE_ID, Arc::new(NoDependencyAdapter))
        .expect("zero-dependency prepared rule binds");
    let named = bound
        .validate_named(ValidationValue::Typed(&String::from("target")), &[])
        .expect("empty named dependencies are valid");
    let ordered = bound
        .validate(
            ValidationValue::Typed(&String::from("target")),
            &BoundValidationContext::new(&[]),
        )
        .expect("empty ordered context is valid");
    assert_eq!(named, ordered);
}

#[test]
fn test_validate_named_one_dependency_preserves_paths_and_shape_errors() {
    let bound = SINGLE_DESCRIPTOR.bind(RULE_ID, 0, &[]).expect("signature binds");
    let path = ValidationPath::root().with_field("item");
    let dependency = NamedValidationDependency::new("item", ValidationValue::Text("dependency value"));
    assert_eq!(
        bound
            .validate_named(ValidationValue::Text("target"), &[dependency])
            .expect("one named dependency binds"),
        ValidationOutcome::Valid,
    );

    let dependency = NamedValidationDependency::new("item", ValidationValue::Text("dependency value")).with_path(&path);
    assert_eq!(
        bound
            .validate_named(ValidationValue::Text("target"), &[dependency])
            .expect("one named dependency with a path binds"),
        ValidationOutcome::Valid,
    );

    let wrong = NamedValidationDependency::new("item", ValidationValue::Typed(&7_u64)).with_path(&path);
    let error = bound
        .validate_named(ValidationValue::Text("target"), &[wrong])
        .expect_err("dependency type mismatch is preserved");
    assert_eq!(error.kind(), ExecutionErrorKind::DependencyTypeMismatch);
    assert_eq!(error.rule_id(), Some(RULE_ID));
    assert_eq!(error.dependency(), Some("item"));
    assert_eq!(error.path(), &path);

    let error = bound
        .validate_named(ValidationValue::Text("target"), &[])
        .expect_err("missing dependency is preserved");
    assert_eq!(error.kind(), ExecutionErrorKind::MissingDependencyBinding);
    assert_eq!(error.rule_id(), Some(RULE_ID));
    assert_eq!(error.dependency(), Some("item"));
}
