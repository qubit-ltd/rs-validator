use std::any::TypeId;
use std::error::Error;
use std::sync::Arc;

use qubit_validator::ArgumentReader;
use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidationContext;
use qubit_validator::DependencySpec;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PathSegment;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::SkipReason;
use qubit_validator::SkippedValidation;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorSignature;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationCodeError;
use qubit_validator::ViolationDraft;
use qubit_validator::ViolationParam;

fn valid(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    struct Valid;
    impl PreparedValidator for Valid {
        fn validate(
            &self,
            _: ValidationValue<'_>,
            _: &BoundValidationContext<'_>,
        ) -> Result<PreparedOutcome, ExecutionError> {
            Ok(PreparedOutcome::Valid)
        }
    }
    Ok(Arc::new(Valid))
}

fn invalid_empty(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    struct Invalid;
    impl PreparedValidator for Invalid {
        fn validate(
            &self,
            _: ValidationValue<'_>,
            _: &BoundValidationContext<'_>,
        ) -> Result<PreparedOutcome, ExecutionError> {
            Ok(PreparedOutcome::Invalid(vec![]))
        }
    }
    Ok(Arc::new(Invalid))
}

fn skipped_missing(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    struct Skipped;
    impl PreparedValidator for Skipped {
        fn validate(
            &self,
            _: ValidationValue<'_>,
            _: &BoundValidationContext<'_>,
        ) -> Result<PreparedOutcome, ExecutionError> {
            Ok(PreparedOutcome::Skipped {
                reason: SkipReason::MissingOptional,
                prerequisites: vec![Violation::new(
                    ValidatorId::new("test.prerequisite"),
                    ViolationCode::new("test.failed"),
                )],
            })
        }
    }
    Ok(Arc::new(Skipped))
}

fn invalid_nonempty(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    struct Invalid;
    impl PreparedValidator for Invalid {
        fn validate(
            &self,
            _: ValidationValue<'_>,
            _: &BoundValidationContext<'_>,
        ) -> Result<PreparedOutcome, ExecutionError> {
            Ok(PreparedOutcome::Invalid(vec![ViolationDraft::new(ViolationCode::new(
                "test.invalid",
            ))]))
        }
    }
    Ok(Arc::new(Invalid))
}

fn skipped_prerequisite(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    struct Skipped;
    impl PreparedValidator for Skipped {
        fn validate(
            &self,
            _: ValidationValue<'_>,
            _: &BoundValidationContext<'_>,
        ) -> Result<PreparedOutcome, ExecutionError> {
            Ok(PreparedOutcome::Skipped {
                reason: SkipReason::FailedPrerequisite,
                prerequisites: vec![Violation::new(
                    ValidatorId::new("test.prerequisite"),
                    ViolationCode::new("test.failed"),
                )],
            })
        }
    }
    Ok(Arc::new(Skipped))
}

fn prepared_error(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Err(BindError::new(BindErrorKind::InvalidPattern))
}

static TEXT_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], valid)];
static TEXT_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(TEXT_SIGNATURES);

fn registration(id: &'static str) -> ValidatorRegistration {
    ValidatorRegistration::new(
        ValidatorId::new(id),
        &TEXT_DESCRIPTOR,
        RegistrationSource::new("test", "next_contract_tests", "next_contract_tests.rs", 1),
    )
}

#[test]
fn argument_reader_covers_typed_and_optional_parameters() {
    let args = [
        NamedValidationArgument::new("count", ValidationArgument::Unsigned(7)),
        NamedValidationArgument::new("signed", ValidationArgument::Integer(8)),
        NamedValidationArgument::new("name", ValidationArgument::String("value")),
        NamedValidationArgument::new("enabled", ValidationArgument::Bool(true)),
    ];
    let mut reader = ArgumentReader::new(&args).unwrap();
    assert_eq!(reader.required_u32("count").unwrap(), 7);
    assert_eq!(reader.optional_u32("signed").unwrap(), Some(8));
    assert_eq!(reader.required_str("name").unwrap(), "value");
    assert_eq!(reader.optional_bool("enabled").unwrap(), Some(true));
    assert_eq!(reader.optional_u32("missing").unwrap(), None);
    reader.finish().unwrap();
    assert!(format!("{reader:?}").contains("argument_count"));
}

#[test]
fn argument_reader_reports_missing_type_range_and_unknown_errors() {
    let mut reader = ArgumentReader::new(&[]).unwrap();
    assert_eq!(
        reader.required_u32("count").unwrap_err().kind(),
        BindErrorKind::MissingParameter
    );
    assert_eq!(
        reader.required_str("name").unwrap_err().kind(),
        BindErrorKind::MissingParameter
    );

    let args = [
        NamedValidationArgument::new("count", ValidationArgument::String("x")),
        NamedValidationArgument::new("enabled", ValidationArgument::Unsigned(1)),
    ];
    let mut reader = ArgumentReader::new(&args).unwrap();
    assert_eq!(
        reader.required_u32("count").unwrap_err().kind(),
        BindErrorKind::ParameterTypeMismatch
    );
    assert_eq!(
        reader.optional_bool("enabled").unwrap_err().kind(),
        BindErrorKind::ParameterTypeMismatch
    );

    let args = [NamedValidationArgument::new("count", ValidationArgument::Integer(-1))];
    let mut reader = ArgumentReader::new(&args).unwrap();
    assert_eq!(
        reader.required_u32("count").unwrap_err().kind(),
        BindErrorKind::ParameterOutOfRange
    );

    let args = [NamedValidationArgument::new(
        "count",
        ValidationArgument::Unsigned(u128::from(u32::MAX) + 1),
    )];
    let mut reader = ArgumentReader::new(&args).unwrap();
    assert_eq!(
        reader.required_u32("count").unwrap_err().kind(),
        BindErrorKind::ParameterOutOfRange
    );

    let args = [NamedValidationArgument::new("other", ValidationArgument::Bool(false))];
    let reader = ArgumentReader::new(&args).unwrap();
    assert_eq!(reader.finish().unwrap_err().kind(), BindErrorKind::UnknownParameter);
    assert_eq!(
        ArgumentReader::new(&[args[0], args[0]]).unwrap_err().kind(),
        BindErrorKind::DuplicateParameter
    );
}

#[test]
fn values_paths_and_input_shapes_are_safe() {
    let number = 4_u32;
    let text = ValidationValue::Text("secret");
    let typed = ValidationValue::Typed(&number);
    let missing = ValidationValue::Missing;
    assert_eq!(text.input_type(), Some(InputType::Text));
    assert_eq!(typed.input_type(), Some(InputType::of::<u32>()));
    assert_eq!(missing.input_type(), None);
    assert_eq!(text.as_text(), Some("secret"));
    assert_eq!(typed.typed::<u32>(), Some(&number));
    assert_eq!(typed.typed::<u64>(), None);
    assert!(missing.is_missing());
    assert!(format!("{text:?}").contains("redacted"));
    assert!(InputType::Text.accepts(text));
    assert!(!InputType::Text.accepts(typed));
    assert!(InputType::of::<u32>().accepts(typed));

    let path = ValidationPath::root()
        .with_field("user")
        .with_index(2)
        .with_map_entry(3)
        .with_map_key()
        .with_map_value();
    assert_eq!(path.render(), "user[2].<map-entry:3>.<map-key>.<map-value>");
    assert_eq!(path.as_segments()[0], PathSegment::Field("user".into()));
    assert_eq!(path.to_string(), "<validation-path>");
    assert!(format!("{path:?}").contains("segment_count"));
}

#[test]
fn context_checks_paths_shapes_and_dependencies() {
    let value = 9_u32;
    let values = [
        ValidationValue::Typed(&value),
        ValidationValue::Missing,
        ValidationValue::Text("text"),
    ];
    let context = BoundValidationContext::new(&values);
    assert_eq!(context.typed::<u32>(0).unwrap(), &value);
    assert_eq!(context.optional_typed::<u32>(1).unwrap(), None);
    assert_eq!(context.text(2).unwrap(), "text");
    assert_eq!(context.dependency_path(0).unwrap(), &ValidationPath::root());
    assert_eq!(context.value(0).unwrap().typed::<u32>(), Some(&value));
    assert_eq!(
        context.typed::<u64>(0).unwrap_err().kind(),
        ExecutionErrorKind::DependencyTypeMismatch
    );
    assert_eq!(
        context.text(1).unwrap_err().kind(),
        ExecutionErrorKind::MissingRequiredDependencyValue
    );
    assert_eq!(
        context.optional_typed::<u32>(2).unwrap_err().kind(),
        ExecutionErrorKind::DependencyTypeMismatch
    );
    assert_eq!(
        context.value(3).unwrap_err().kind(),
        ExecutionErrorKind::AdapterContractViolation
    );
    assert_eq!(
        context.dependency_path(3).unwrap_err().kind(),
        ExecutionErrorKind::AdapterContractViolation
    );

    let path = ValidationPath::root().with_field("dependency");
    let paths = [path.clone(), ValidationPath::root(), ValidationPath::root()];
    let with_paths = BoundValidationContext::new_with_paths(&values, &paths).unwrap();
    assert_eq!(with_paths.dependency_path(0).unwrap(), &path);
    assert_eq!(
        BoundValidationContext::new_with_paths(&values, &[]).unwrap_err().kind(),
        ExecutionErrorKind::AdapterContractViolation
    );
}

#[test]
fn descriptor_binding_and_bound_validation_cover_contract_errors() {
    let args = [NamedValidationArgument::new("unused", ValidationArgument::Bool(false))];
    assert_eq!(
        TEXT_DESCRIPTOR
            .bind(ValidatorId::new("test.rule"), 9, &[])
            .unwrap_err()
            .kind(),
        BindErrorKind::InvalidSelection
    );
    assert_eq!(
        TEXT_DESCRIPTOR
            .bind_for(ValidatorId::new("test.rule"), InputType::of::<u32>(), &[])
            .unwrap_err()
            .kind(),
        BindErrorKind::UnsupportedInput
    );
    assert!(
        TEXT_DESCRIPTOR
            .bind_for(ValidatorId::new("test.rule"), InputType::Text, &[])
            .is_ok()
    );
    static ERROR_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepared_error)];
    let error_descriptor = ValidatorDescriptor::new(ERROR_SIGNATURES);
    assert_eq!(
        error_descriptor
            .bind(ValidatorId::new("test.rule"), 0, &args)
            .unwrap_err()
            .kind(),
        BindErrorKind::InvalidPattern
    );

    static INVALID_SIGNATURES: &[ValidatorSignature] = &[
        ValidatorSignature::new(InputType::Text, &[], valid),
        ValidatorSignature::new(InputType::Text, &[], valid),
    ];
    assert_eq!(
        ValidatorDescriptor::try_new(INVALID_SIGNATURES).unwrap_err().kind(),
        BindErrorKind::AmbiguousSignature
    );

    static DEPENDENCIES: &[DependencySpec] = &[
        DependencySpec::new("same", InputType::Text, false),
        DependencySpec::new("same", InputType::Text, true),
    ];
    static BAD_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, DEPENDENCIES, valid)];
    assert_eq!(
        ValidatorDescriptor::try_new(BAD_SIGNATURES).unwrap_err().kind(),
        BindErrorKind::InvalidDeclaration
    );

    let bound = TEXT_DESCRIPTOR.bind(ValidatorId::new("test.rule"), 0, &[]).unwrap();
    assert_eq!(bound.input_type(), InputType::Text);
    assert!(bound.dependency_specs().is_empty());
    assert_eq!(bound.rule_id(), ValidatorId::new("test.rule"));
    assert_eq!(
        bound
            .validate(ValidationValue::Text("ok"), &BoundValidationContext::new(&[]))
            .unwrap(),
        ValidationOutcome::Valid
    );
    assert_eq!(
        bound
            .validate(ValidationValue::Missing, &BoundValidationContext::new(&[]))
            .unwrap_err()
            .kind(),
        ExecutionErrorKind::InputTypeMismatch
    );
}

#[test]
fn bound_validation_checks_dependency_contracts_and_outcome_contracts() {
    static DEPENDENCIES: &[DependencySpec] = &[
        DependencySpec::new("required", InputType::Text, false),
        DependencySpec::new("optional", InputType::of::<u32>(), true),
    ];
    static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, DEPENDENCIES, valid)];
    static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
    let bound = DESCRIPTOR.bind(ValidatorId::new("test.rule"), 0, &[]).unwrap();

    let number = 1_u32;
    let missing_optional = [ValidationValue::Text("dependency"), ValidationValue::Missing];
    assert_eq!(
        bound
            .validate(
                ValidationValue::Text("ok"),
                &BoundValidationContext::new(&missing_optional)
            )
            .unwrap(),
        ValidationOutcome::Valid
    );
    let wrong = [ValidationValue::Typed(&number), ValidationValue::Missing];
    let error = bound
        .validate(ValidationValue::Text("ok"), &BoundValidationContext::new(&wrong))
        .unwrap_err();
    assert_eq!(error.kind(), ExecutionErrorKind::DependencyTypeMismatch);
    assert_eq!(error.rule_id(), Some(ValidatorId::new("test.rule")));
    let missing = [ValidationValue::Missing, ValidationValue::Missing];
    assert_eq!(
        bound
            .validate(ValidationValue::Text("ok"), &BoundValidationContext::new(&missing))
            .unwrap_err()
            .kind(),
        ExecutionErrorKind::MissingRequiredDependencyValue
    );
    assert_eq!(
        bound
            .validate(ValidationValue::Text("ok"), &BoundValidationContext::new(&[]))
            .unwrap_err()
            .kind(),
        ExecutionErrorKind::AdapterContractViolation
    );

    static INVALID_SIGNATURE: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], invalid_empty)];
    let invalid = ValidatorDescriptor::new(INVALID_SIGNATURE)
        .bind(ValidatorId::new("test.rule"), 0, &[])
        .unwrap();
    assert_eq!(
        invalid
            .validate(ValidationValue::Text("ok"), &BoundValidationContext::new(&[]))
            .unwrap_err()
            .kind(),
        ExecutionErrorKind::AdapterContractViolation
    );

    static SKIPPED_SIGNATURE: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], skipped_missing)];
    let skipped = ValidatorDescriptor::new(SKIPPED_SIGNATURE)
        .bind(ValidatorId::new("test.rule"), 0, &[])
        .unwrap();
    assert_eq!(
        skipped
            .validate(ValidationValue::Text("ok"), &BoundValidationContext::new(&[]))
            .unwrap_err()
            .kind(),
        ExecutionErrorKind::AdapterContractViolation
    );
}

#[test]
fn errors_reports_violations_and_registries_expose_structured_data() {
    let source = RegistrationSource::new("crate", "module", "file.rs", 42);
    assert_eq!(
        (source.crate_name(), source.module_path(), source.file(), source.line()),
        ("crate", "module", "file.rs", 42)
    );
    let error = BindError::new(BindErrorKind::MissingDependencyDeclaration)
        .with_rule(ValidatorId::new("test.rule"))
        .with_parameter("minimum")
        .with_dependency("other");
    assert_eq!(error.rule_id(), Some(ValidatorId::new("test.rule")));
    assert_eq!(error.parameter(), Some("minimum"));
    assert_eq!(error.dependency(), Some("other"));
    assert!(format!("{error:?}").contains("has_parameter"));
    assert!(error.to_string().contains("missing dependency declaration"));

    let execution = ExecutionError::new(ExecutionErrorKind::ExternalFailure)
        .with_rule(ValidatorId::new("test.rule"))
        .with_path(ValidationPath::root().with_field("secret"))
        .with_source(std::io::Error::other("private"));
    assert_eq!(execution.rule_id(), Some(ValidatorId::new("test.rule")));
    assert!(execution.source().is_some());
    assert!(!execution.to_string().contains("private"));
    assert!(format!("{execution:?}").contains("ExecutionError"));

    let skipped = SkippedValidation::new(3, ValidationPath::root(), SkipReason::MissingOptional);
    assert_eq!(
        (skipped.occurrence(), skipped.reason()),
        (3, SkipReason::MissingOptional)
    );
    assert_eq!(skipped.path(), &ValidationPath::root());
    let violation = Violation::new(ValidatorId::new("test.rule"), ViolationCode::new("test.invalid"))
        .with_path(ValidationPath::root().with_field("name"))
        .with_param("ok", ViolationParam::Bool(false))
        .with_param("count", ViolationParam::Unsigned(2));
    assert_eq!(violation.code().as_str(), "test.invalid");
    assert_eq!(violation.rule_id(), ValidatorId::new("test.rule"));
    assert_eq!(violation.path().render(), "name");
    assert_eq!(violation.params().len(), 2);
    assert_eq!(violation.to_string(), "test.invalid");
    assert!(format!("{violation:?}").contains("Violation"));

    let mut report = ValidationReport::default();
    assert!(report.is_valid());
    assert!(report.push_skipped(skipped));
    assert!(report.is_valid());
    report.mark_truncated();
    assert!(!report.is_valid());
    assert!(report.is_truncated());
    assert!(report.push_violation(violation));
    assert_eq!(report.violations().len(), 1);
    assert!(report.to_string().contains("1 violation"));
    assert!(format!("{report:?}").contains("violation_count"));

    let registry = ValidatorRegistry::from_registrations([registration("test.registry")]).unwrap();
    assert!(format!("{registry:?}").contains("ValidatorRegistry"));
    let reference = registration("test.reference");
    let copied = ValidatorRegistry::from_registrations([&reference]).unwrap();
    assert!(copied.get("test.reference").is_some());
    let bound = registry.bind("test.registry", InputType::Text, &[]).unwrap();
    assert_eq!(bound.rule_id(), ValidatorId::new("test.registry"));
    assert_eq!(
        registry.bind("missing", InputType::Text, &[]).unwrap_err().kind(),
        BindErrorKind::MissingRule
    );
}

#[test]
fn debug_and_error_trait_surfaces_are_covered() {
    let dependency = DependencySpec::new("dependency", InputType::Text, true);
    assert_eq!(
        (dependency.name(), dependency.input(), dependency.optional()),
        ("dependency", InputType::Text, true)
    );
    assert!(format!("{dependency:?}").contains("DependencySpec"));

    let signatures: &'static [ValidatorSignature] = Box::leak(Box::new([
        ValidatorSignature::new(InputType::Text, &[], valid),
        ValidatorSignature::new(InputType::of::<u32>(), &[], valid),
    ]));
    let descriptor = ValidatorDescriptor::new(signatures);
    assert!(format!("{descriptor:?}").contains("signature_count"));
    assert_eq!(signatures[0].input(), InputType::Text);
    assert!(signatures[0].dependencies().is_empty());
    assert!(format!("{:?}", signatures[0]).contains("ValidatorSignature"));

    let context = BoundValidationContext::new(&[]);
    assert!(format!("{context:?}").contains("slot_count"));
    let bound = descriptor.bind(ValidatorId::new("test.rule"), 0, &[]).unwrap();
    assert!(format!("{bound:?}").contains("BoundValidator"));

    let error = ExecutionError::new(ExecutionErrorKind::ExternalFailure).with_source(std::io::Error::other("private"));
    assert!(Error::source(&error).is_some());

    for segment in [
        PathSegment::Field("field".into()),
        PathSegment::Index(1),
        PathSegment::MapEntry(2),
        PathSegment::MapKey,
        PathSegment::MapValue,
    ] {
        assert!(!format!("{segment:?}").is_empty());
    }
}

#[test]
fn bound_validator_accepts_valid_nonempty_and_prerequisite_outcomes() {
    static INVALID: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], invalid_nonempty)];
    static PREREQUISITE: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], skipped_prerequisite)];
    let invalid = ValidatorDescriptor::new(INVALID)
        .bind(ValidatorId::new("test.rule"), 0, &[])
        .unwrap();
    let prerequisite = ValidatorDescriptor::new(PREREQUISITE)
        .bind(ValidatorId::new("test.rule"), 0, &[])
        .unwrap();
    assert!(matches!(
        invalid.validate(ValidationValue::Text("ok"), &BoundValidationContext::new(&[])).unwrap(),
        ValidationOutcome::Invalid(issues) if issues.len() == 1 && issues[0].rule_id() == ValidatorId::new("test.rule")
    ));
    assert!(matches!(
        prerequisite.validate(ValidationValue::Text("ok"), &BoundValidationContext::new(&[])).unwrap(),
        ValidationOutcome::Skipped { reason: SkipReason::FailedPrerequisite, prerequisites } if prerequisites.len() == 1 && prerequisites[0].rule_id() == ValidatorId::new("test.prerequisite")
    ));
}

#[test]
fn protocol_and_enum_display_values_are_stable() {
    for kind in [
        BindErrorKind::UnknownParameter,
        BindErrorKind::DuplicateParameter,
        BindErrorKind::MissingParameter,
        BindErrorKind::ParameterTypeMismatch,
        BindErrorKind::ParameterOutOfRange,
        BindErrorKind::InvalidBounds,
        BindErrorKind::InvalidPattern,
        BindErrorKind::MissingRule,
        BindErrorKind::UnsupportedInput,
        BindErrorKind::MissingDependencyDeclaration,
        BindErrorKind::UnknownDependencyDeclaration,
        BindErrorKind::DependencyTypeMismatch,
        BindErrorKind::UnreadablePath,
        BindErrorKind::AmbiguousSignature,
        BindErrorKind::UnsupportedConstraint,
        BindErrorKind::MissingFeature,
        BindErrorKind::InvalidDeclaration,
        BindErrorKind::InvalidSelection,
    ] {
        assert!(!kind.to_string().is_empty());
    }
    for kind in [
        ExecutionErrorKind::InputTypeMismatch,
        ExecutionErrorKind::MissingRequiredDependencyValue,
        ExecutionErrorKind::DependencyTypeMismatch,
        ExecutionErrorKind::PropertyReadFailed,
        ExecutionErrorKind::TraversalLimit,
        ExecutionErrorKind::AdapterContractViolation,
        ExecutionErrorKind::ExternalFailure,
    ] {
        assert!(!kind.to_string().is_empty());
    }
    assert_eq!(ViolationCode::try_new("").unwrap_err(), ViolationCodeError::Empty);
    assert_eq!(
        ViolationCode::try_new("a.").unwrap_err(),
        ViolationCodeError::EmptySegment
    );
    assert_eq!(
        ViolationCode::try_new("a..b").unwrap_err(),
        ViolationCodeError::EmptySegment
    );
    assert_eq!(
        ViolationCode::try_new("1bad").unwrap_err(),
        ViolationCodeError::InvalidSegment
    );
    assert_eq!(
        ViolationCode::try_new("a-b").unwrap_err(),
        ViolationCodeError::InvalidSegment
    );
    assert_eq!(ViolationCode::try_new("a.b_2").unwrap().as_str(), "a.b_2");
    assert_eq!(InputType::of::<u32>(), InputType::Typed(TypeId::of::<u32>()));
}
