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
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorRegistryError;
use qubit_validator::ValidatorSignature;
use qubit_validator::register_validator;

fn prepare(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    struct AcceptAll;

    impl PreparedValidator for AcceptAll {
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

register_validator!(id = "test.inventory.duplicate", descriptor = &DESCRIPTOR);
register_validator!(id = "test.inventory.duplicate", descriptor = &DESCRIPTOR);

#[test]
fn test_global_registry_caches_duplicate_registration_error() {
    let first = ValidatorRegistry::try_global().expect_err("duplicate inventory IDs are rejected");
    assert!(matches!(first, ValidatorRegistryError::DuplicateId { .. }));

    let second = ValidatorRegistry::try_global().expect_err("the failed result is cached");
    assert!(matches!(second, ValidatorRegistryError::DuplicateId { .. }));

    let panic =
        std::panic::catch_unwind(ValidatorRegistry::global).expect_err("global panics when initialization failed");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .expect("global panic includes a string message");
    assert!(message.contains("duplicate validator ID"));
}
