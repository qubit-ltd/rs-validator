// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::convert::Infallible;
use std::sync::Arc;

use qubit_validator::BindError;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::Validator;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorSignature;

struct AcceptAll;

impl Validator<str> for AcceptAll {
    type Error = Infallible;

    fn validate(&self, _: &str, _: &()) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn prepare_valid(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(qubit_validator::prepare_text_validator(AcceptAll, |_| {
        qubit_validator::ViolationDraft::new(qubit_validator::ViolationCode::new("test.invalid"))
    }))
}

static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare_valid)];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);

#[test]
fn test_registration_preserves_source_and_declared_signature() {
    let source = RegistrationSource::new("consumer", "consumer::rules", "rules.rs", 17);
    let registration = ValidatorRegistration::new(ValidatorId::new("consumer.rule"), &DESCRIPTOR, source);

    assert_eq!(registration.id().as_str(), "consumer.rule");
    assert_eq!(registration.source().crate_name(), "consumer");
    assert_eq!(registration.source().module_path(), "consumer::rules");
    assert_eq!(registration.source().file(), "rules.rs");
    assert_eq!(registration.source().line(), 17);
    assert!(std::ptr::eq(registration.descriptor().signatures(), SIGNATURES));
}
