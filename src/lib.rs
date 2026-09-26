// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Type-safe validation traits and immutable validator registries.

#![deny(missing_docs)]
#![deny(unsafe_code)]

mod argument;
mod binding;
mod internal;
mod registry;
mod report;
mod validator;
mod validator_id;
mod validator_id_error;
mod validator_registry_error;

#[cfg(feature = "inventory")]
#[doc(hidden)]
pub mod __private {
    pub use inventory;
}

pub use argument::NamedValidationArgument;
pub use argument::ValidationArgument;
pub use binding::ArgumentReader;
pub use binding::BindError;
pub use binding::BindErrorKind;
pub use binding::BoundValidationContext;
pub use binding::BoundValidator;
pub use binding::DependencySpec;
pub use binding::ExecutionError;
pub use binding::ExecutionErrorKind;
pub use binding::InputType;
pub use binding::PrepareFn;
pub use binding::PreparedOutcome;
pub use binding::PreparedValidator;
pub use binding::ValidationOutcome;
pub use binding::ValidationValue;
pub use binding::ViolationDraft;
pub use binding::prepare_contextual_text_validator;
pub use binding::prepare_contextual_typed_validator;
pub use binding::prepare_text_validator;
pub use binding::prepare_typed_validator;
pub use registry::RegistrationSource;
pub use registry::ValidatorDescriptor;
pub use registry::ValidatorRegistration;
#[cfg(feature = "inventory")]
#[doc(hidden)]
pub use registry::ValidatorRegistrationFactory;
pub use registry::ValidatorRegistry;
pub use registry::ValidatorSignature;
pub use report::FailureId;
pub use report::PathSegment;
pub use report::RecordedOutcome;
pub use report::SkipReason;
pub use report::SkippedValidation;
pub use report::ValidationLimits;
pub use report::ValidationOutcomeError;
pub use report::ValidationPath;
pub use report::ValidationReport;
pub use report::Violation;
pub use report::ViolationCode;
pub use report::ViolationCodeError;
pub use report::ViolationParam;
pub use validator::Validator;
pub use validator_id::ValidatorId;
pub use validator_id_error::ValidatorIdError;
pub use validator_registry_error::ValidatorRegistryError;
