// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Validation reporting primitives.
mod path_segment;
mod skip_reason;
mod skipped_validation;
mod validation_limits;
mod validation_path;
mod validation_report;
mod violation;
mod violation_code;
mod violation_code_error;
mod violation_param;
pub use path_segment::PathSegment;
pub use skip_reason::SkipReason;
pub use skipped_validation::SkippedValidation;
pub use validation_limits::ValidationLimits;
pub use validation_path::ValidationPath;
pub use validation_report::ValidationReport;
pub use violation::Violation;
pub use violation_code::ViolationCode;
pub use violation_code_error::ViolationCodeError;
pub use violation_param::ViolationParam;
