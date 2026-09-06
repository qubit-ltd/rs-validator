use std::sync::Arc;

use qubit_validator::BindError;
use qubit_validator::BoundValidationContext;
use qubit_validator::ExecutionError;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::RuleOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorSignature;

fn prepare(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    struct AlwaysValid;

    impl PreparedValidator for AlwaysValid {
        fn validate(
            &self,
            _: ValidationValue<'_>,
            _: &BoundValidationContext<'_>,
        ) -> Result<RuleOutcome, ExecutionError> {
            Ok(RuleOutcome::Valid)
        }
    }

    Ok(Arc::new(AlwaysValid))
}

static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare)];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);

fn registration(id: &'static str, file: &'static str) -> ValidatorRegistration {
    ValidatorRegistration::new(
        ValidatorId::new(id),
        &DESCRIPTOR,
        RegistrationSource::new("test", "registry", file, 1),
    )
}

#[test]
fn local_registry_owns_and_queries_registrations() {
    let first = registration("example.valid", "first.rs");
    let registry = ValidatorRegistry::from_registrations([first]).expect("valid registry");

    assert_eq!(
        registry.get("example.valid").map(|entry| entry.id().as_str()),
        Some("example.valid")
    );
    assert!(registry.get("missing").is_none());
    assert_eq!(registry.registrations().len(), 1);
}

#[test]
fn local_registry_rejects_duplicate_ids() {
    let first = registration("example.valid", "first.rs");
    let second = registration("example.valid", "second.rs");
    let error = ValidatorRegistry::from_registrations([first, second]).expect_err("duplicate ID");

    assert!(error.to_string().contains("example.valid"));
    assert!(error.to_string().contains("first.rs"));
    assert!(error.to_string().contains("second.rs"));
}

#[test]
fn empty_registry_has_no_rules() {
    let registry = ValidatorRegistry::empty();

    assert!(registry.registrations().is_empty());
    assert!(registry.get("missing").is_none());
}
