// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

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

fn prepare(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    struct AlwaysValid;

    impl PreparedValidator for AlwaysValid {
        fn input_type(&self) -> InputType {
            InputType::Text
        }
        fn dependency_specs(&self) -> &'static [qubit_validator::DependencySpec] {
            &[]
        }

        fn validate(
            &self,
            _: ValidationValue<'_>,
            _: &BoundValidationContext<'_>,
        ) -> Result<PreparedOutcome, ExecutionError> {
            Ok(PreparedOutcome::Valid)
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
fn test_local_registry_owns_and_queries_registrations() {
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
fn test_registry_sorts_registrations_for_binary_search() {
    let registry = ValidatorRegistry::from_registrations([
        registration("example.middle", "middle.rs"),
        registration("example.zulu", "zulu.rs"),
        registration("example.alpha", "alpha.rs"),
    ])
    .expect("unique registrations form a registry");

    let ids = registry
        .registrations()
        .iter()
        .map(|entry| entry.id().as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids, ["example.alpha", "example.middle", "example.zulu"]);
    for id in ids {
        assert_eq!(registry.get(id).map(|entry| entry.id().as_str()), Some(id));
    }
    assert!(registry.get("example.beta").is_none());
}

#[test]
fn test_local_registry_rejects_duplicate_ids() {
    let first = registration("example.valid", "first.rs");
    let second = registration("example.valid", "second.rs");
    let error = ValidatorRegistry::from_registrations([first, second]).expect_err("duplicate ID");

    assert!(error.to_string().contains("example.valid"));
    assert!(error.to_string().contains("first.rs"));
    assert!(error.to_string().contains("second.rs"));
}

#[test]
fn test_empty_registry_has_no_rules() {
    let registry = ValidatorRegistry::empty();

    assert!(registry.registrations().is_empty());
    assert!(registry.get("missing").is_none());
}

#[test]
fn test_registry_accepts_borrowed_registration_and_debug_is_available() {
    let registration = registration("example.borrowed", "borrowed.rs");
    let registry = ValidatorRegistry::from_registrations([&registration])
        .expect("borrowed registration is copied into the registry");

    assert!(registry.get("example.borrowed").is_some());
    assert!(format!("{registry:?}").contains("example.borrowed"));
}
