use std::sync::Arc;

use qubit_validator::BindError;
use qubit_validator::BoundValidationContext;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
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
use qubit_validator::prepare_text_validator;
use qubit_validator::prepare_typed_validator;

#[derive(Debug, thiserror::Error)]
#[error("value must be at least three")]
struct TooSmall;

struct AtLeastThree;

impl Validator<i64> for AtLeastThree {
    type Error = TooSmall;

    fn validate(&self, value: &i64, _: &()) -> Result<(), Self::Error> {
        if *value >= 3 { Ok(()) } else { Err(TooSmall) }
    }
}

fn prepare_at_least_three(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(prepare_typed_validator(AtLeastThree, |_| {
        ViolationDraft::new(ViolationCode::new("number.too_small"))
            .with_path(ValidationPath::root().with_field("value"))
            .with_param("minimum", ViolationParam::Signed(3))
            .with_param("message_key", ViolationParam::Token("validation.number.too_small"))
    }))
}

static TYPED_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::of::<i64>(),
    &[],
    prepare_at_least_three,
)];
static TYPED_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(TYPED_SIGNATURES);

#[derive(Debug, thiserror::Error)]
#[error("text must not be blank")]
struct BlankText;

struct NonBlankText;

impl Validator<str> for NonBlankText {
    type Error = BlankText;

    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        if value.trim().is_empty() {
            Err(BlankText)
        } else {
            Ok(())
        }
    }
}

fn prepare_non_blank_text(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(prepare_text_validator(NonBlankText, |_| {
        ViolationDraft::new(ViolationCode::new("text.blank"))
    }))
}

static TEXT_SIGNATURES: &[ValidatorSignature] =
    &[ValidatorSignature::new(InputType::Text, &[], prepare_non_blank_text)];
static TEXT_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(TEXT_SIGNATURES);

#[test]
fn test_typed_adapter_accepts_valid_input() {
    let validator = TYPED_DESCRIPTOR
        .bind(ValidatorId::new("test.at_least_three"), 0, &[], &[])
        .expect("typed validator binds");

    let outcome = validator
        .validate(ValidationValue::Typed(&3_i64), &BoundValidationContext::new(&[]))
        .expect("valid typed input executes");

    assert_eq!(outcome, ValidationOutcome::Valid);
}

#[test]
fn test_typed_adapter_maps_domain_error_to_structured_violation() {
    let rule_id = ValidatorId::new("test.at_least_three");
    let validator = TYPED_DESCRIPTOR
        .bind(rule_id, 0, &[], &[])
        .expect("typed validator binds");

    let outcome = validator
        .validate(ValidationValue::Typed(&2_i64), &BoundValidationContext::new(&[]))
        .expect("invalid typed input remains a validation outcome");
    let ValidationOutcome::Invalid(violations) = outcome else {
        panic!("expected one structured violation");
    };
    let [violation] = violations.as_slice() else {
        panic!("expected exactly one structured violation");
    };

    assert_eq!(violation.rule_id(), rule_id);
    assert_eq!(violation.code().as_str(), "number.too_small");
    assert_eq!(violation.to_string(), "number.too_small");
    assert_eq!(violation.path().render(), "value");
    assert_eq!(violation.params().get("minimum"), Some(&ViolationParam::Signed(3)));
    assert_eq!(
        violation.params().get("message_key"),
        Some(&ViolationParam::Token("validation.number.too_small"))
    );
}

#[test]
fn test_typed_adapter_rejects_text_input() {
    let validator = prepare_typed_validator(AtLeastThree, |_| {
        ViolationDraft::new(ViolationCode::new("number.too_small"))
    });

    let error = validator
        .validate(ValidationValue::Text("3"), &BoundValidationContext::new(&[]))
        .expect_err("text must not reach a typed validator");

    assert_eq!(error.kind(), ExecutionErrorKind::InputTypeMismatch);
}

#[test]
fn test_text_adapter_accepts_non_blank_input() {
    let validator = TEXT_DESCRIPTOR
        .bind(ValidatorId::new("test.non_blank_text"), 0, &[], &[])
        .expect("text validator binds");

    let outcome = validator
        .validate(ValidationValue::Text("value"), &BoundValidationContext::new(&[]))
        .expect("valid text input executes");

    assert_eq!(outcome, ValidationOutcome::Valid);
}

#[test]
fn test_text_adapter_rejects_blank_input_without_exposing_it() {
    let validator = TEXT_DESCRIPTOR
        .bind(ValidatorId::new("test.non_blank_text"), 0, &[], &[])
        .expect("text validator binds");
    let rejected_value = " \t\n";

    let outcome = validator
        .validate(ValidationValue::Text(rejected_value), &BoundValidationContext::new(&[]))
        .expect("blank text remains a validation outcome");
    let ValidationOutcome::Invalid(violations) = &outcome else {
        panic!("expected one text violation");
    };
    let [violation] = violations.as_slice() else {
        panic!("expected exactly one text violation");
    };

    assert_eq!(violation.code().as_str(), "text.blank");
    assert_eq!(violation.path(), &ValidationPath::root());
    assert!(!format!("{outcome:?}").contains(rejected_value));
}

#[test]
fn test_text_adapter_rejects_typed_input() {
    let validator = prepare_text_validator(NonBlankText, |_| ViolationDraft::new(ViolationCode::new("text.blank")));

    let error = validator
        .validate(ValidationValue::Typed(&3_i64), &BoundValidationContext::new(&[]))
        .expect_err("typed input must not reach a text validator");

    assert_eq!(error.kind(), ExecutionErrorKind::InputTypeMismatch);
}
