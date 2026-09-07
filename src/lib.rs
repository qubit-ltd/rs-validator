// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Type-safe validation traits and immutable validator registries.

#![deny(missing_docs)]
#![deny(unsafe_code)]

mod argument;
mod binding;
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

pub use argument::{NamedValidationArgument, ValidationArgument};
pub use binding::{ArgumentReader, BindError, BindErrorKind, BoundValidationContext,
    BoundValidator, DependencySpec, ExecutionError, ExecutionErrorKind, InputType,
    PrepareFn, PreparedOutcome, PreparedValidator, RuleOutcome, ValidationOutcome,
    ValidationValue, ViolationDraft, prepare_text_validator, prepare_typed_validator};
pub use registry::{RegistrationSource, ValidatorDescriptor, ValidatorRegistration,
    ValidatorRegistry, ValidatorSignature};
pub use report::{PathSegment, SkipReason, SkippedValidation, ValidationPath,
    ValidationLimits, ValidationReport, Violation, ViolationCode, ViolationCodeError, ViolationParam};
pub use validator::Validator;
pub use validator_id::ValidatorId;
pub use validator_id_error::ValidatorIdError;
#[cfg(feature = "inventory")]
#[doc(hidden)]
pub use registry::ValidatorRegistrationFactory;
pub use validator_registry_error::ValidatorRegistryError;
