// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! The next-generation validation contracts.

mod bind_error;
mod bind_error_kind;
mod execution_error;
mod execution_error_kind;
mod path_segment;
mod rule_outcome;
mod skip_reason;
mod skipped_validation;
mod validation_path;
mod validation_report;
mod validator;
mod violation;
mod violation_code;
mod violation_code_error;
mod violation_param;

pub use bind_error::BindError;
pub use bind_error_kind::BindErrorKind;
pub use execution_error::ExecutionError;
pub use execution_error_kind::ExecutionErrorKind;
pub use path_segment::PathSegment;
pub use rule_outcome::RuleOutcome;
pub use skip_reason::SkipReason;
pub use skipped_validation::SkippedValidation;
pub use validation_path::ValidationPath;
pub use validation_report::ValidationReport;
pub use validator::Validator;
pub use violation::Violation;
pub use violation_code::ViolationCode;
pub use violation_code_error::ViolationCodeError;
pub use violation_param::ViolationParam;
