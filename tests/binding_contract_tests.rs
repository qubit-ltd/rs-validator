use std::convert::Infallible;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use qubit_validator::next::ArgumentReader;
use qubit_validator::next::BoundValidationContext;
use qubit_validator::next::InputType;
use qubit_validator::next::PreparedValidator;
use qubit_validator::next::RuleOutcome;
use qubit_validator::next::ValidationValue;
use qubit_validator::next::ValidatorDescriptor;
use qubit_validator::next::ValidatorRegistration;
use qubit_validator::next::ValidatorSignature;
use qubit_validator::NamedValidationArgument;
use qubit_validator::RegistrationSource;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidatorId;

#[test]
fn argument_reader_rejects_bad_names_types_and_ranges() {
    let args = [
        NamedValidationArgument::new("min", ValidationArgument::String("3")),
        NamedValidationArgument::new("min", ValidationArgument::Unsigned(1)),
    ];
    assert_eq!(
        ArgumentReader::new(&args).unwrap_err().kind(),
        qubit_validator::next::BindErrorKind::DuplicateParameter
    );

    let args = [NamedValidationArgument::new(
        "miin",
        ValidationArgument::Unsigned(3),
    )];
    let reader = ArgumentReader::new(&args).unwrap();
    assert_eq!(
        reader.finish().unwrap_err().kind(),
        qubit_validator::next::BindErrorKind::UnknownParameter
    );

    let args = [NamedValidationArgument::new(
        "min",
        ValidationArgument::Integer(-1),
    )];
    let mut reader = ArgumentReader::new(&args).unwrap();
    assert_eq!(
        reader.required_u32("min").unwrap_err().kind(),
        qubit_validator::next::BindErrorKind::ParameterOutOfRange
    );
}

struct CountingAdapter {
    calls: Arc<AtomicUsize>,
}

impl PreparedValidator for CountingAdapter {
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<RuleOutcome, qubit_validator::next::ExecutionError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert!(matches!(value, ValidationValue::Text("ok")));
        Ok(RuleOutcome::Valid)
    }
}

fn prepare_counting(
    _: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, qubit_validator::next::BindError> {
    Ok(Arc::new(CountingAdapter {
        calls: Arc::new(AtomicUsize::new(0)),
    }))
}

static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    &[],
    prepare_counting,
)];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
static REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("test.counting"),
    &DESCRIPTOR,
    RegistrationSource::new("test", "binding", "binding_contract_tests.rs", 1),
);

#[test]
fn bound_validator_can_be_cloned_and_uses_one_prepared_instance() {
    let bound = REGISTRATION.descriptor().bind(0, &[]).unwrap();
    let clone = bound.clone();
    clone
        .validate(
            ValidationValue::Text("ok"),
            &BoundValidationContext::new(&[]),
        )
        .unwrap();
    bound
        .validate(
            ValidationValue::Text("ok"),
            &BoundValidationContext::new(&[]),
        )
        .unwrap();
}

#[test]
fn dependency_context_preserves_optional_missing_and_rejects_wrong_shapes() {
    let owner = 7_u64;
    let values = [ValidationValue::Typed(&owner)];
    let context = BoundValidationContext::new(&values);
    assert_eq!(context.typed::<u64>(0).unwrap(), &owner);
    assert!(context.typed::<u32>(0).is_err());
}

#[test]
fn registration_descriptor_has_a_stable_signature() {
    assert_eq!(REGISTRATION.descriptor().signatures().len(), 1);
    assert_eq!(REGISTRATION.source().crate_name(), "test");
}

#[derive(Default)]
struct _Infallible;
impl qubit_validator::next::Validator<str> for _Infallible {
    type Error = Infallible;
    fn validate(&self, _: &str, _: &()) -> Result<(), Self::Error> {
        Ok(())
    }
}
