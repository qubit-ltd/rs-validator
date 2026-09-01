use std::convert::Infallible;

use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationContext;
use qubit_validator::ValidationDependency;
use qubit_validator::Validator;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorExecutionError;

#[derive(Default)]
struct Minimum;

impl Validator<u32> for Minimum {
    type Error = Infallible;

    fn validate(&mut self, value: &u32, context: &ValidationContext<'_>) -> Result<(), Self::Error> {
        let ValidationArgument::Unsigned(minimum) = context.argument("minimum").expect("minimum argument") else {
            unreachable!("fixture uses an unsigned minimum")
        };
        assert!(*value as u128 >= minimum);
        assert_eq!(
            context
                .dependency("tenant")
                .and_then(|value| value.downcast_ref::<u64>()),
            Some(&7)
        );
        Ok(())
    }
}

#[test]
fn test_descriptor_validates_typed_value_with_context() {
    let arguments = [NamedValidationArgument::new("minimum", ValidationArgument::Unsigned(3))];
    let tenant = 7_u64;
    let dependencies = [ValidationDependency::new("tenant", &tenant)];
    let context = ValidationContext::new(&arguments, &dependencies);
    let descriptor = ValidatorDescriptor::of::<Minimum, u32>();

    descriptor.validate(&5_u32, &context).expect("valid value");
}

#[test]
fn test_descriptor_rejects_wrong_erased_type() {
    let descriptor = ValidatorDescriptor::of::<Minimum, u32>();
    let error = descriptor
        .validate(&5_u64, &ValidationContext::default())
        .expect_err("wrong type must fail");

    assert!(matches!(error, ValidatorExecutionError::TypeMismatch { .. }));
    assert!(error.to_string().contains("u32"));
}

#[derive(Default)]
struct Rejecting;

impl Validator<String> for Rejecting {
    type Error = std::io::Error;

    fn validate(&mut self, _value: &String, _context: &ValidationContext<'_>) -> Result<(), Self::Error> {
        Err(std::io::Error::other("rejected by fixture"))
    }
}

#[test]
fn test_descriptor_exposes_identity_and_preserves_source_error() {
    let descriptor = ValidatorDescriptor::of::<Rejecting, String>();

    assert_eq!(descriptor.validator_type_id(), std::any::TypeId::of::<Rejecting>());
    assert_eq!(descriptor.validator_type_name(), std::any::type_name::<Rejecting>());
    assert_eq!(descriptor.value_type_id(), std::any::TypeId::of::<String>());
    assert_eq!(descriptor.value_type_name(), std::any::type_name::<String>());
    assert!(format!("{descriptor:?}").contains("Rejecting"));

    let error = descriptor
        .validate(&String::from("value"), &ValidationContext::default())
        .expect_err("fixture rejects every value");
    assert!(matches!(error, ValidatorExecutionError::ValidationFailed { .. }));
    assert!(error.to_string().contains("rejected by fixture"));
}

#[test]
fn test_context_exposes_entries_and_absent_lookups() {
    let arguments = [NamedValidationArgument::new("enabled", ValidationArgument::Bool(true))];
    let owner = String::from("alice");
    let dependencies = [ValidationDependency::new("owner", &owner)];
    let context = ValidationContext::new(&arguments, &dependencies);

    assert_eq!(context.arguments(), &arguments);
    assert_eq!(context.argument("enabled"), Some(ValidationArgument::Bool(true)));
    assert_eq!(context.argument("missing"), None);
    assert_eq!(context.dependencies().len(), 1);
    assert_eq!(context.dependencies()[0].path(), "owner");
    assert!(std::ptr::eq(context.dependencies()[0].value(), &owner));
    assert_eq!(
        context
            .dependency("owner")
            .and_then(|value| value.downcast_ref::<String>()),
        Some(&owner)
    );
    assert!(context.dependency("missing").is_none());
}
