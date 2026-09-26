// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Validation reporting primitives.
mod failure_id;
mod path_segment;
mod recorded_outcome;
mod skip_reason;
mod skipped_validation;
mod validation_limits;
mod validation_outcome_error;
mod validation_path;
mod validation_report;
mod violation;
mod violation_code;
mod violation_code_error;
mod violation_param;
pub use failure_id::FailureId;
pub(crate) use failure_id::next_report_id;
pub use path_segment::PathSegment;
pub use recorded_outcome::RecordedOutcome;
pub use skip_reason::SkipReason;
pub use skipped_validation::SkippedValidation;
pub use validation_limits::ValidationLimits;
pub use validation_outcome_error::ValidationOutcomeError;
pub use validation_path::ValidationPath;
pub use validation_report::ValidationReport;
pub use violation::Violation;
pub use violation_code::ViolationCode;
pub use violation_code_error::ViolationCodeError;
pub use violation_param::ViolationParam;
