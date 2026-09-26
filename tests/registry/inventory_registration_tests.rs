// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![cfg(feature = "inventory")]

use std::sync::Arc;

use qubit_validator::BindError;
use qubit_validator::BoundValidationContext;
use qubit_validator::ExecutionError;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorSignature;
use qubit_validator::register_validator;

struct AlwaysValid;

impl PreparedValidator for AlwaysValid {
    fn validate(
        &self,
        _: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        Ok(PreparedOutcome::Valid)
    }
}

fn prepare(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(AlwaysValid))
}

static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare)];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);

register_validator!(id = "test.inventory.global", descriptor = &DESCRIPTOR);

const SHARED_REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("test.inventory.shared"),
    &DESCRIPTOR,
    RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
);
register_validator!(registration = SHARED_REGISTRATION);

#[test]
fn test_inventory_registration_is_available_from_global_registry() {
    let registry = ValidatorRegistry::try_global().expect("inventory registry is valid");
    assert!(registry.get("test.inventory.global").is_some());
    assert!(std::ptr::eq(registry, ValidatorRegistry::global()));
}

#[test]
fn test_inventory_registration_reuses_the_local_registration_value() {
    let registry = ValidatorRegistry::try_global().expect("inventory registry is valid");
    let registered = registry
        .get("test.inventory.shared")
        .expect("shared registration is submitted");
    assert_eq!(registered.id(), SHARED_REGISTRATION.id());
    assert!(std::ptr::eq(registered.descriptor(), SHARED_REGISTRATION.descriptor()));
    assert_eq!(registered.source(), SHARED_REGISTRATION.source());
}

#[test]
fn test_local_registry_accepts_static_registration_references() {
    let registration = registry_registration();
    let registry = ValidatorRegistry::from_registrations([&registration])
        .expect("reference registration is copied into the local registry");
    assert_eq!(
        registry.get("test.inventory.local").unwrap().id().as_str(),
        "test.inventory.local"
    );
}

fn registry_registration() -> ValidatorRegistration {
    ValidatorRegistration::new(
        ValidatorId::new("test.inventory.local"),
        &DESCRIPTOR,
        RegistrationSource::new("test", "inventory", "inventory_registry_tests.rs", 1),
    )
}
