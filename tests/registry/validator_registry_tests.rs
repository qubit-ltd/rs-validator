// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;
use std::sync::Arc;

use qubit_validator::ArgumentReader;
use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidationContext;
use qubit_validator::DependencySpec;
use qubit_validator::ExecutionError;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorRegistryError;
use qubit_validator::ValidatorSignature;

fn prepare(_reader: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
    struct AlwaysValid;

    impl PreparedValidator for AlwaysValid {
        fn input_type(&self) -> InputType {
            InputType::Text
        }
        fn dependency_specs(&self) -> &'static [DependencySpec] {
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
    let expected = vec![first.source(), second.source()];
    for input in [[first, second], [second, first]] {
        let error = ValidatorRegistry::from_registrations(input).expect_err("duplicate ID");
        match error {
            ValidatorRegistryError::DuplicateId { id, sources } => {
                assert_eq!(id, "example.valid");
                assert_eq!(sources, expected);
            }
            _ => panic!("expected duplicate ID metadata"),
        }
    }
}

#[test]
fn test_registry_preserves_declaration_dependency_without_formatting_it() {
    static DEPENDENCIES: &[DependencySpec] = &[
        DependencySpec::new("private_dependency_slot", InputType::Text, false),
        DependencySpec::new("private_dependency_slot", InputType::Text, false),
    ];
    static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, DEPENDENCIES, prepare)];
    static INVALID: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
    let id = ValidatorId::new("test.invalid_descriptor");
    let location = RegistrationSource::new("test", "registry", "invalid.rs", 7);
    let registration = ValidatorRegistration::new(id, &INVALID, location);
    let error = ValidatorRegistry::from_registrations([registration])
        .expect_err("duplicate dependency declarations must be rejected");
    let cloned = error.clone();
    for error in [&error, &cloned] {
        match error {
            ValidatorRegistryError::InvalidDescriptor {
                id: actual_id,
                registration_source,
                kind,
                dependency,
            } => {
                assert_eq!(*actual_id, id);
                assert_eq!(*registration_source, location);
                assert_eq!(*kind, BindErrorKind::InvalidDeclaration);
                assert_eq!(dependency.as_deref(), Some("private_dependency_slot"));
            }
            _ => panic!("expected invalid descriptor metadata"),
        }
        assert!(!error.to_string().contains("private_dependency_slot"));
        assert!(!format!("{error:?}").contains("private_dependency_slot"));
        assert!(Error::source(error).is_none());
    }
}

#[test]
fn test_registry_invalid_descriptor_without_dependency_keeps_none() {
    static EMPTY: ValidatorDescriptor = ValidatorDescriptor::new(&[]);
    let registration = ValidatorRegistration::new(
        ValidatorId::new("test.empty_descriptor"),
        &EMPTY,
        RegistrationSource::new("test", "registry", "empty.rs", 1),
    );
    let error = ValidatorRegistry::from_registrations([registration])
        .expect_err("a descriptor requires at least one signature");
    match error {
        ValidatorRegistryError::InvalidDescriptor { kind, dependency, .. } => {
            assert_eq!(kind, BindErrorKind::InvalidDeclaration);
            assert_eq!(dependency, None);
        }
        _ => panic!("expected invalid descriptor metadata"),
    }
}

#[test]
fn test_empty_registry_has_no_rules() {
    let registry = ValidatorRegistry::empty();

    assert!(registry.registrations().is_empty());
    assert!(registry.get("missing").is_none());
}

#[test]
fn test_missing_rule_has_no_resolved_rule_id() {
    let registry = ValidatorRegistry::empty();
    let error = registry
        .bind("example.missing", InputType::Text, &[])
        .expect_err("the requested rule is not registered");

    assert_eq!(error.kind(), BindErrorKind::MissingRule);
    assert_eq!(error.rule_id(), None);
}

#[test]
fn test_registry_bind_rejects_unknown_parameter_when_factory_ignores_reader() {
    let rule = ValidatorId::new("example.valid");
    let registry =
        ValidatorRegistry::from_registrations([registration(rule.as_str(), "valid.rs")]).expect("valid registry");
    let arguments = [NamedValidationArgument::new("unknown", ValidationArgument::Unsigned(1))];

    let error = registry
        .bind(rule.as_str(), InputType::Text, &arguments)
        .expect_err("unknown parameter must be rejected");

    assert_eq!(error.kind(), BindErrorKind::UnknownParameter);
    assert_eq!(error.rule_id(), Some(rule));
}

#[test]
fn test_registry_accepts_borrowed_registration_and_debug_is_available() {
    let registration = registration("example.borrowed", "borrowed.rs");
    let registry = ValidatorRegistry::from_registrations([&registration])
        .expect("borrowed registration is copied into the registry");

    assert!(registry.get("example.borrowed").is_some());
    assert!(format!("{registry:?}").contains("example.borrowed"));
}
