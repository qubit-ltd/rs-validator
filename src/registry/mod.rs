// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Validator registration and registry.
mod registration_source;
mod validator_descriptor;
mod validator_registration;
#[cfg(feature = "inventory")]
mod validator_registration_factory;
mod validator_registry;
mod validator_signature;
pub use registration_source::RegistrationSource;
pub use validator_descriptor::ValidatorDescriptor;
pub use validator_registration::ValidatorRegistration;
#[cfg(feature = "inventory")]
pub use validator_registration_factory::ValidatorRegistrationFactory;
pub use validator_registry::ValidatorRegistry;
pub use validator_signature::ValidatorSignature;

pub(crate) use crate::binding::BindError;
pub(crate) use crate::binding::BindErrorKind;
pub(crate) use crate::binding::BoundValidator;
pub(crate) use crate::binding::DependencySpec;
pub(crate) use crate::binding::InputType;
pub(crate) use crate::binding::PrepareFn;
