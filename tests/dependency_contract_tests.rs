use std::sync::Arc;

use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidationContext;
use qubit_validator::DependencySpec;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
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

#[test]
fn test_bind_rejects_dependency_declarations_in_the_wrong_slot_order() {
    let declared = [DEPENDENCIES[1], DEPENDENCIES[0]];

    let error = DESCRIPTOR
        .bind(RULE_ID, 0, &[], &declared)
        .expect_err("swapped dependency slots must fail during binding");

    assert_eq!(error.kind(), BindErrorKind::DependencyOrderMismatch);
    assert_eq!(error.dependency(), Some("maximum"));
}

#[test]
fn test_bind_accepts_dependency_declarations_in_signature_order() {
    DESCRIPTOR
        .bind(RULE_ID, 0, &[], DEPENDENCIES)
        .expect("dependency slots in signature order must bind");
}

#[test]
fn test_validate_reports_missing_dependency_name_rule_and_path() {
    let bound = DESCRIPTOR
        .bind(RULE_ID, 0, &[], DEPENDENCIES)
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
        .bind(RULE_ID, 0, &[], DEPENDENCIES)
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
