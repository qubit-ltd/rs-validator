// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![deny(unreachable_patterns)]

use qubit_validator::InputType;
use qubit_validator::RegistrationSource;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;

static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(&[]);

#[test]
fn test_registration_constructors_are_public() {
    let source = RegistrationSource::new("consumer", "consumer::rules", "rules.rs", 17);
    let registration = ValidatorRegistration::new(ValidatorId::new("consumer.non_blank"), &DESCRIPTOR, source);

    assert_eq!(registration.id().as_str(), "consumer.non_blank");
    assert_eq!(registration.source().crate_name(), "consumer");
    assert_eq!(registration.source().line(), 17);
}

#[test]
fn test_public_enum_matching_allows_future_variants() {
    let input = InputType::Text;

    let label = match input {
        InputType::Text => "text",
        InputType::Typed(_) => "typed",
        _ => "other",
    };

    assert_eq!(label, "text");
}
