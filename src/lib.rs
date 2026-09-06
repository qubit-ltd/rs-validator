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
mod validation_context;
mod validation_dependency;
mod validator;
mod validator_descriptor;
mod validator_execution_error;
mod validator_id;
mod validator_id_error;
mod validator_registration;
mod validator_registration_factory;
mod validator_registry;
mod validator_registry_error;

/// The next-generation typed validation contracts and structured outcomes.
pub mod next;

#[doc(hidden)]
pub mod __private {
    pub use inventory;
}

pub use named_validation_argument::NamedValidationArgument;
pub use registration_source::RegistrationSource;
pub use validation_argument::ValidationArgument;
pub use validation_context::ValidationContext;
pub use validation_dependency::ValidationDependency;
pub use validator::Validator;
pub use validator_descriptor::ValidatorDescriptor;
pub use validator_execution_error::ValidatorExecutionError;
pub use validator_id::ValidatorId;
pub use validator_id_error::ValidatorIdError;
pub use validator_registration::ValidatorRegistration;
#[doc(hidden)]
pub use validator_registration_factory::ValidatorRegistrationFactory;
pub use validator_registry::ValidatorRegistry;
pub use validator_registry_error::ValidatorRegistryError;
