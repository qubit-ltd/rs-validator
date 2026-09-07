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
pub(crate) use crate::binding::{BindError, BindErrorKind, BoundValidator, InputType};
pub(crate) use crate::binding::{DependencySpec, PrepareFn};
