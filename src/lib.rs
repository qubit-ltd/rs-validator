// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Type-safe validation traits and immutable validator registries.

#![deny(missing_docs)]
#![deny(unsafe_code)]

mod named_validation_argument;
mod registration_source;
mod validation_argument;
mod validator;
mod validator_id;
mod validator_id_error;
#[cfg(feature = "inventory")]
mod validator_registration_factory;
mod validator_registry_error;

/// Typed validation contracts, structured outcomes, and registries.
pub mod next;

#[cfg(feature = "inventory")]
#[doc(hidden)]
pub mod __private {
    pub use inventory;
}

pub use named_validation_argument::NamedValidationArgument;
pub use next::ArgumentReader;
pub use next::BindError;
pub use next::BindErrorKind;
pub use next::BoundValidationContext;
pub use next::BoundValidator;
pub use next::DependencySpec;
pub use next::ExecutionError;
pub use next::ExecutionErrorKind;
pub use next::InputType;
pub use next::PathSegment;
pub use next::PrepareFn;
pub use next::PreparedValidator;
pub use next::RuleOutcome;
pub use next::SkipReason;
pub use next::SkippedValidation;
pub use next::ValidationPath;
pub use next::ValidationReport;
pub use next::ValidationValue;
pub use next::ValidatorDescriptor;
pub use next::ValidatorRegistration;
pub use next::ValidatorRegistry;
pub use next::ValidatorSignature;
pub use next::Violation;
pub use next::ViolationCode;
pub use next::ViolationCodeError;
pub use next::ViolationParam;
pub use registration_source::RegistrationSource;
pub use validation_argument::ValidationArgument;
pub use validator::Validator;
pub use validator_id::ValidatorId;
pub use validator_id_error::ValidatorIdError;
#[cfg(feature = "inventory")]
#[doc(hidden)]
pub use validator_registration_factory::ValidatorRegistrationFactory;
pub use validator_registry_error::ValidatorRegistryError;
