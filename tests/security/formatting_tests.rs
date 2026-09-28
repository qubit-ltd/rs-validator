// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::convert::Infallible;
use std::sync::Arc;

use qubit_validator::ArgumentReader;
use qubit_validator::BindError;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PathSegment;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::SkipReason;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;
use qubit_validator::Validator;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorSignature;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;

struct Accept;

impl Validator<str> for Accept {
    type Error = Infallible;

    fn validate(&self, _: &str, _: &()) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn prepare(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(qubit_validator::prepare_text_validator(Accept, |never| match never {}))
}

static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare)];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
static REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("test.accept"),
    &DESCRIPTOR,
    RegistrationSource::new("test", "security::formatting_tests", "formatting_tests.rs", 1),
);

#[test]
fn test_public_formatters_expose_safe_metadata_and_reader_decodes_booleans() {
    let arguments = [NamedValidationArgument::new("enabled", ValidationArgument::Bool(true))];
    let mut reader = ArgumentReader::new(&arguments).expect("unique argument names");
    assert!(!format!("{reader:?}").contains("true"));
    assert_eq!(reader.optional_bool("enabled").unwrap(), Some(true));
    assert_eq!(reader.optional_bool("absent").unwrap(), None);
    reader.finish().unwrap();

    let signature_debug = format!("{:?}", SIGNATURES[0]);
    assert!(signature_debug.contains("Text"));
    let bound = DESCRIPTOR
        .bind(ValidatorId::new("test.accept"), 0, &[])
        .expect("declared signature binds");
    assert!(format!("{bound:?}").contains("test.accept"));
    assert_eq!(bound.input_type(), InputType::Text);
    let _: &RegistrationSource = &REGISTRATION.source();
    assert!(!format!("{:?}", PathSegment::Field("secret-field")).contains("secret-field"));

    let mut report = ValidationReport::default();
    let outcome = report
        .record_outcome(
            0,
            ValidationPath::root(),
            ValidationOutcome::invalid(vec![Violation::new(
                ValidatorId::new("test.accept"),
                ViolationCode::new("text.invalid"),
            )])
            .unwrap(),
        )
        .unwrap();
    assert!(outcome.complete());
    report
        .record_outcome(1, ValidationPath::root(), ValidationOutcome::missing_optional())
        .unwrap();
    assert!(format!("{report}").contains("1 violation"));
    assert!(!format!("{report:?}").contains("text.invalid"));
    assert_eq!(report.skipped()[0].reason(), SkipReason::MissingOptional);

    assert_eq!(
        bound
            .validate(
                qubit_validator::ValidationValue::Text("value"),
                &qubit_validator::BoundValidationContext::new(&[]),
            )
            .unwrap(),
        ValidationOutcome::Valid
    );
}
