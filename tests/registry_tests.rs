use std::convert::Infallible;

use qubit_validator::RegistrationSource;
use qubit_validator::ValidationContext;
use qubit_validator::Validator;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorIdError;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorRegistryError;
use qubit_validator::register_validator;

register_validator!(id = "example.global", validator = AlwaysValid, value = String,);

#[derive(Default)]
struct AlwaysValid;

impl Validator<String> for AlwaysValid {
    type Error = Infallible;

    fn validate(&mut self, _value: &String, _context: &ValidationContext<'_>) -> Result<(), Self::Error> {
        Ok(())
    }
}

static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::of::<AlwaysValid, String>();
static FIRST: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("example.valid"),
    &DESCRIPTOR,
    RegistrationSource::new("fixture", "first", "first.rs", 1),
);
static SECOND: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("example.valid"),
    &DESCRIPTOR,
    RegistrationSource::new("fixture", "second", "second.rs", 2),
);

#[test]
fn test_validator_id_rejects_invalid_segments() {
    assert_eq!(ValidatorId::new("example.Valid").as_str(), "example.Valid");
    assert_eq!(ValidatorId::try_new(""), Err(ValidatorIdError::Empty));
    assert_eq!(
        ValidatorId::try_new("example.bad-id"),
        Err(ValidatorIdError::InvalidSegment)
    );
    assert_eq!(
        ValidatorId::try_new("example..bad"),
        Err(ValidatorIdError::EmptySegment)
    );
    assert_eq!(ValidatorId::try_new("example."), Err(ValidatorIdError::EmptySegment));
    assert_eq!(
        ValidatorId::try_new("9example.bad"),
        Err(ValidatorIdError::InvalidSegment)
    );
}

#[test]
fn test_local_registry_owns_and_queries_registrations() {
    let registry = ValidatorRegistry::from_registrations([&FIRST]).expect("valid registry");
    assert_eq!(registry.get("example.valid").map(|entry| entry.id()), Some(FIRST.id()));
    assert!(registry.get("missing").is_none());
    assert_eq!(registry.registrations().len(), 1);
    assert_eq!(
        registry.registrations()[0].descriptor().value_type_id(),
        std::any::TypeId::of::<String>()
    );
    let source = registry.registrations()[0].source();
    assert_eq!(source.crate_name(), "fixture");
    assert_eq!(source.module_path(), "first");
    assert_eq!(source.file(), "first.rs");
    assert_eq!(source.line(), 1);

    let empty = ValidatorRegistry::empty();
    assert!(empty.registrations().is_empty());
    assert!(empty.get("example.valid").is_none());
}

#[test]
fn test_registry_rejects_duplicate_ids() {
    let error = ValidatorRegistry::from_registrations([&FIRST, &SECOND]).expect_err("duplicate ID");
    assert!(matches!(error, ValidatorRegistryError::DuplicateId { .. }));
    assert!(error.to_string().contains("example.valid"));
}

#[test]
fn test_global_registry_collects_registered_validator() {
    let registry = ValidatorRegistry::try_global().expect("valid linked registry");
    let global = ValidatorRegistry::global();

    assert!(std::ptr::eq(registry, global));
    let registration = global.get("example.global").expect("linked validator");
    assert_eq!(registration.id().as_str(), "example.global");
    assert_eq!(
        registration.descriptor().validator_type_id(),
        std::any::TypeId::of::<AlwaysValid>()
    );
    assert_eq!(registration.source().crate_name(), env!("CARGO_PKG_NAME"));
}

#[test]
fn test_global_registry_initialization_is_unique_across_threads() {
    let addresses = (0..8)
        .map(|_| {
            std::thread::spawn(|| {
                ValidatorRegistry::try_global().expect("valid linked registry") as *const ValidatorRegistry as usize
            })
        })
        .map(|thread| thread.join().expect("registry thread must complete"))
        .collect::<Vec<_>>();

    assert!(addresses.iter().all(|address| *address == addresses[0]));
}
