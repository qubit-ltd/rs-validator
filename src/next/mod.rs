// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! The next-generation validation contracts.

mod argument_reader;
mod bind_error;
mod bind_error_kind;
mod bound_validation_context;
mod bound_validator;
mod dependency_spec;
mod execution_error;
mod execution_error_kind;
mod input_type;
mod path_segment;
mod prepared_validator;
mod rule_outcome;
mod skip_reason;
mod skipped_validation;
mod validation_path;
mod validation_report;
mod validation_value;
mod validator_descriptor;
mod validator_registration;
mod validator_registry;
mod validator_signature;
mod violation;
mod violation_code;
mod violation_code_error;
mod violation_param;

pub use argument_reader::ArgumentReader;
pub use bind_error::BindError;
pub use bind_error_kind::BindErrorKind;
pub use bound_validation_context::BoundValidationContext;
pub use bound_validator::BoundValidator;
pub use dependency_spec::DependencySpec;
pub use execution_error::ExecutionError;
pub use execution_error_kind::ExecutionErrorKind;
pub use input_type::InputType;
pub use path_segment::PathSegment;
pub use prepared_validator::PrepareFn;
pub use prepared_validator::PreparedValidator;
pub use rule_outcome::RuleOutcome;
pub use skip_reason::SkipReason;
pub use skipped_validation::SkippedValidation;
pub use validation_path::ValidationPath;
pub use validation_report::ValidationReport;
pub use validation_value::ValidationValue;
pub use validator_descriptor::ValidatorDescriptor;
pub use validator_registration::ValidatorRegistration;
pub use validator_registry::ValidatorRegistry;
pub use validator_signature::ValidatorSignature;
pub use violation::Violation;
pub use violation_code::ViolationCode;
pub use violation_code_error::ViolationCodeError;
pub use violation_param::ViolationParam;
