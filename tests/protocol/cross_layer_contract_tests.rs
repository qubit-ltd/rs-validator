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
use qubit_validator::DependencySpec;
use qubit_validator::ExecutionError;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorSignature;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;

struct Rejecting;

impl PreparedValidator for Rejecting {
    fn input_type(&self) -> InputType {
        InputType::Text
    }

    fn dependency_specs(&self) -> &'static [DependencySpec] {
        &[]
    }

    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        match value {
            ValidationValue::Text("ok") => Ok(PreparedOutcome::Valid),
            ValidationValue::Text(_) => Ok(PreparedOutcome::Invalid(vec![
                ViolationDraft::new(ViolationCode::new("text.rejected"))
                    .with_path(ValidationPath::root().with_field("value")),
            ])),
            _ => unreachable!("bound validation enforces the declared text input"),
        }
    }
}

fn prepare(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(Rejecting))
}

static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare)];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
static REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("test.cross_layer"),
    &DESCRIPTOR,
    RegistrationSource::new("test", "cross_layer", "cross_layer_contract_tests.rs", 1),
);

#[test]
fn test_registry_validation_report_and_prerequisite_flow_preserves_failure_identity() {
    let registry = ValidatorRegistry::from_registrations([REGISTRATION]).expect("registration is valid");
    let bound = registry
        .bind("test.cross_layer", InputType::Text, &[])
        .expect("text signature binds");
    let outcome = bound
        .validate(
            ValidationValue::Text("private input"),
            &BoundValidationContext::new(&[]),
        )
        .expect("domain rejection becomes a validation outcome");
    let mut report = ValidationReport::new();
    let prefix = ValidationPath::root().with_field("profile");
    let receipt = report
        .record_outcome(0, prefix.clone(), outcome)
        .expect("validation outcome records");

    assert!(receipt.complete());
    assert_eq!(receipt.failure_ids().len(), 1);
    let failure_id = receipt.failure_ids()[0];
    let failure = report.failure(failure_id).expect("receipt identifies retained failure");
    assert_eq!(failure.rule_id(), ValidatorId::new("test.cross_layer"));
    assert_eq!(failure.code().as_str(), "text.rejected");
    assert_eq!(failure.path(), &prefix.with_field("value"));
    assert!(!format!("{report:?}").contains("private input"));

    let skipped = report
        .record_outcome(
            1,
            ValidationPath::root().with_field("dependent"),
            ValidationOutcome::failed_prerequisite(vec![failure_id]).expect("one prerequisite failure is well-formed"),
        )
        .expect("dependent skip references the retained failure");
    assert!(skipped.complete());
    assert_eq!(report.failure_count(), 1);
    assert_eq!(report.skipped().len(), 1);
    assert_eq!(report.skipped()[0].prerequisites(), &[failure_id]);
}
