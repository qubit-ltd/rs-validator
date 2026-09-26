// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use qubit_validator::ArgumentReader;
use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidationContext;
use qubit_validator::DependencySpec;
use qubit_validator::ExecutionError;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::NamedValidationDependency;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorSignature;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;
use qubit_validator::prepare_text_validator;
#[cfg(feature = "inventory")]
use qubit_validator::register_validator;

#[derive(Debug)]
struct BlankText;

impl fmt::Display for BlankText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("text must not be blank")
    }
}

impl Error for BlankText {}

struct NonBlank;

impl Validator<str> for NonBlank {
    type Error = BlankText;

    fn validate(&self, value: &str, _context: &()) -> Result<(), Self::Error> {
        if value.trim().is_empty() {
            Err(BlankText)
        } else {
            Ok(())
        }
    }
}

fn prepare_non_blank(params: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    ArgumentReader::new(params)?.finish()?;
    Ok(prepare_text_validator(NonBlank, |_| {
        ViolationDraft::new(ViolationCode::new("text.blank"))
    }))
}

static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare_non_blank)];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);

#[cfg(feature = "inventory")]
register_validator!(id = "text.non_blank.global", descriptor = &DESCRIPTOR);

static DEPENDENCIES: &[DependencySpec] = &[
    DependencySpec::new("minimum", InputType::Text, false),
    DependencySpec::new("maximum", InputType::Text, false),
];

struct CheckDependencySlots;

impl PreparedValidator for CheckDependencySlots {
    fn input_type(&self) -> InputType {
        InputType::Text
    }
    fn dependency_specs(&self) -> &'static [qubit_validator::DependencySpec] {
        &[]
    }

    fn validate(
        &self,
        _: ValidationValue<'_>,
        context: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        let valid = context.value(0)?.as_text() == Some("minimum") && context.value(1)?.as_text() == Some("maximum");
        Ok(if valid {
            PreparedOutcome::Valid
        } else {
            PreparedOutcome::Invalid(Vec::new())
        })
    }
}

fn prepare_dependency_slots(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(CheckDependencySlots))
}

static DEPENDENCY_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    DEPENDENCIES,
    prepare_dependency_slots,
)];
static DEPENDENCY_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(DEPENDENCY_SIGNATURES);

fn assert_dependency_specs_come_from_signature() -> Result<(), ExecutionError> {
    let bound = DEPENDENCY_DESCRIPTOR
        .bind(ValidatorId::new("text.dependent"), 0, &[])
        .expect("signature binds");
    assert_eq!(bound.dependency_specs(), DEPENDENCIES);
    let dependencies = [
        NamedValidationDependency::new("maximum", ValidationValue::Text("maximum")),
        NamedValidationDependency::new("minimum", ValidationValue::Text("minimum")),
    ];
    assert_eq!(
        bound.validate_named(ValidationValue::Text("target"), &dependencies)?,
        ValidationOutcome::Valid,
    );
    Ok(())
}

fn assert_parameters_are_consumed_once() -> Result<(), BindError> {
    let args = [NamedValidationArgument::new("limit", ValidationArgument::Unsigned(10))];
    let mut reader = ArgumentReader::new(&args)?;
    assert_eq!(reader.required_u32("limit")?, 10);
    let error = reader
        .required_u32("limit")
        .expect_err("a parameter cannot be read twice");
    assert_eq!(error.kind(), BindErrorKind::ParameterAlreadyConsumed);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let registration = ValidatorRegistration::new(
        ValidatorId::new("text.non_blank"),
        &DESCRIPTOR,
        RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
    );
    let registry = ValidatorRegistry::from_registrations([registration])?;

    assert!(registry.get("text.non_blank").is_some());
    let validator = registry.bind("text.non_blank", InputType::Text, &[])?;
    let context = BoundValidationContext::new(&[]);

    let valid = validator.validate(ValidationValue::Text("Ada"), &context)?;
    assert_eq!(valid, ValidationOutcome::Valid);

    let invalid = validator.validate(ValidationValue::Text("   "), &context)?;
    let ValidationOutcome::Invalid(violations) = invalid else {
        panic!("blank text must be invalid");
    };
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].code(), ViolationCode::new("text.blank"));

    assert_dependency_specs_come_from_signature()?;
    assert_parameters_are_consumed_once()?;

    #[cfg(feature = "inventory")]
    {
        let global = ValidatorRegistry::try_global()?;
        assert!(global.get("text.non_blank.global").is_some());
    }

    Ok(())
}
