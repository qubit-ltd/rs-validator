// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Validation binding primitives.
mod argument_reader;
mod bind_error;
mod bind_error_kind;
mod bound_validation_context;
mod bound_validator;
mod contextual_text_validator_adapter;
mod contextual_typed_validator_adapter;
mod dependency_spec;
mod execution_error;
mod execution_error_kind;
mod input_type;
mod internal;
mod prepared_outcome;
mod prepared_validator;
mod text_validator_adapter;
mod typed_validator_adapter;
mod validation_outcome;
mod validation_value;
mod violation_draft;
pub use argument_reader::ArgumentReader;
pub use bind_error::BindError;
pub use bind_error_kind::BindErrorKind;
pub use bound_validation_context::BoundValidationContext;
pub use bound_validator::BoundValidator;
pub use contextual_text_validator_adapter::prepare_contextual_text_validator;
pub use contextual_typed_validator_adapter::prepare_contextual_typed_validator;
pub use dependency_spec::DependencySpec;
pub use execution_error::ExecutionError;
pub use execution_error_kind::ExecutionErrorKind;
pub use input_type::InputType;
pub use prepared_outcome::PreparedOutcome;
pub use prepared_validator::PrepareFn;
pub use prepared_validator::PreparedValidator;
pub use text_validator_adapter::prepare_text_validator;
pub use typed_validator_adapter::prepare_typed_validator;
pub use validation_outcome::ValidationOutcome;
pub use validation_value::ValidationValue;
pub use violation_draft::ViolationDraft;

pub(crate) use crate::registry::ValidatorSignature;
pub(crate) use crate::report::ValidationPath;
