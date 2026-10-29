// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![cfg(feature = "inventory")]

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_validator::ArgumentReader;
use qubit_validator::BindError;
use qubit_validator::BoundValidationContext;
use qubit_validator::DependencySpec;
use qubit_validator::ExecutionError;
use qubit_validator::InputType;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorRegistryError;
use qubit_validator::ValidatorSignature;
use qubit_validator::register_validator;

fn prepare(_reader: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
    struct AcceptAll;

    impl PreparedValidator for AcceptAll {
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

    Ok(Arc::new(AcceptAll))
}

static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare)];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
static DESCRIPTOR_CALLS: AtomicUsize = AtomicUsize::new(0);

fn counted_descriptor() -> &'static ValidatorDescriptor {
    DESCRIPTOR_CALLS.fetch_add(1, Ordering::Relaxed);
    &DESCRIPTOR
}

register_validator!(id = "test.inventory.duplicate", descriptor = &DESCRIPTOR);
register_validator!(id = "test.inventory.duplicate", descriptor = counted_descriptor());

#[test]
fn test_global_registry_caches_duplicate_registration_error() {
    let first = ValidatorRegistry::try_global().expect_err("duplicate inventory IDs are rejected");
    assert!(matches!(first, ValidatorRegistryError::DuplicateId { .. }));
    assert_eq!(
        DESCRIPTOR_CALLS.load(Ordering::Relaxed),
        1,
        "registration factory runs only during the first initialization"
    );

    let second = ValidatorRegistry::try_global().expect_err("the failed result is cached");
    assert!(matches!(second, ValidatorRegistryError::DuplicateId { .. }));
    assert_eq!(
        DESCRIPTOR_CALLS.load(Ordering::Relaxed),
        1,
        "registration factory runs only during the first initialization"
    );

    let panic =
        std::panic::catch_unwind(ValidatorRegistry::global).expect_err("global panics when initialization failed");
    assert_eq!(
        DESCRIPTOR_CALLS.load(Ordering::Relaxed),
        1,
        "registration factory runs only during the first initialization"
    );
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .expect("global panic includes a string message");
    assert!(message.contains("duplicate validator ID"));
}
