use qubit_validator::next::InputType;
use qubit_validator::next::PreparedValidator;
use qubit_validator::next::RuleOutcome;
use qubit_validator::next::ValidationValue;
use qubit_validator::next::ValidatorDescriptor;
use qubit_validator::next::ValidatorRegistration;
use qubit_validator::next::ValidatorRegistry;
use qubit_validator::next::ValidatorSignature;
use qubit_validator::RegistrationSource;
use qubit_validator::ValidatorId;
use std::sync::Arc;

fn prepare(
    _: &[qubit_validator::NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, qubit_validator::next::BindError> {
    struct Always;
    impl PreparedValidator for Always {
        fn validate(
            &self,
            _: ValidationValue<'_>,
            _: &qubit_validator::next::BoundValidationContext<'_>,
        ) -> Result<RuleOutcome, qubit_validator::next::ExecutionError> {
            Ok(RuleOutcome::Valid)
        }
    }
    Ok(Arc::new(Always))
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
fn duplicate_id_reports_every_source_independent_of_input_order() {
    let one = registration("test.same", "one.rs", &D_A);
    let two = registration("test.same", "two.rs", &D_A);
    let three = registration("test.same", "three.rs", &D_A);
    let error = ValidatorRegistry::from_registrations([three, one, two]).unwrap_err();
    let text = error.to_string();
    assert!(text.contains("one.rs") && text.contains("two.rs") && text.contains("three.rs"));
}

#[test]
fn one_id_can_expose_multiple_input_signatures() {
    let signatures: &'static [ValidatorSignature] = Box::leak(Box::new([
        ValidatorSignature::new(InputType::Text, &[], prepare),
        ValidatorSignature::new(
            InputType::Typed(std::any::TypeId::of::<u32>()),
            &[],
            prepare,
        ),
    ]));
    let descriptor: &'static ValidatorDescriptor =
        Box::leak(Box::new(ValidatorDescriptor::new(signatures)));
    let registry =
        ValidatorRegistry::from_registrations([registration("test.multi", "multi.rs", descriptor)])
            .unwrap();
    assert_eq!(
        registry
            .get("test.multi")
            .unwrap()
            .descriptor()
            .signatures()
            .len(),
        2
    );
}

#[test]
fn duplicate_signatures_are_rejected_when_binding() {
    let signatures: &'static [ValidatorSignature] = Box::leak(Box::new([
        ValidatorSignature::new(InputType::Text, &[], prepare),
        ValidatorSignature::new(InputType::Text, &[], prepare),
    ]));
    let descriptor: &'static ValidatorDescriptor =
        Box::leak(Box::new(ValidatorDescriptor::new(signatures)));
    let error = descriptor.bind_for(InputType::Text, &[]).unwrap_err();
    assert_eq!(
        error.kind(),
        qubit_validator::next::BindErrorKind::AmbiguousSignature
    );
}
