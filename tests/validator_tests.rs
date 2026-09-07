use std::convert::Infallible;
use std::sync::Arc;

use qubit_validator::BindError;
use qubit_validator::BoundValidationContext;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedValidator;
use qubit_validator::RuleOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorSignature;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;

struct Minimum;

impl Validator<u32> for Minimum {
    type Error = Infallible;

    fn validate(&self, value: &u32, _: &()) -> Result<(), Self::Error> {
        assert!(*value >= 3);
        Ok(())
    }
}

#[test]
fn typed_validator_uses_an_immutable_context() {
    Minimum.validate(&5, &()).expect("value is valid");
}

struct Rejecting;

impl PreparedValidator for Rejecting {
    fn validate(
        &self,
        _: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<RuleOutcome, ExecutionError> {
        Ok(RuleOutcome::Invalid(vec![Violation::new(
            ValidatorId::new("test.rejecting"),
            ViolationCode::new("test.rejected"),
        )]))
    }
}

fn prepare_rejecting(
    _: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(Rejecting))
}

static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    &[],
    prepare_rejecting,
)];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);

#[test]
fn prepared_descriptor_preserves_structured_rule_failures() {
    let bound = DESCRIPTOR
        .bind_for(InputType::Text, &[])
        .expect("valid descriptor");
    let outcome = bound
        .validate(
            ValidationValue::Text("value"),
            &BoundValidationContext::new(&[]),
        )
        .expect("adapter execution succeeds");

    assert!(
        matches!(outcome, RuleOutcome::Invalid(violations) if violations[0].code().as_str() == "test.rejected")
    );
}

#[test]
fn prepared_descriptor_rejects_wrong_input_shape() {
    let bound = DESCRIPTOR
        .bind_for(InputType::Text, &[])
        .expect("valid descriptor");
    let error = bound
        .validate(
            ValidationValue::Typed(&5_u32),
            &BoundValidationContext::new(&[]),
        )
        .expect_err("wrong input shape must fail");

    assert_eq!(error.kind(), ExecutionErrorKind::InputTypeMismatch);
}
