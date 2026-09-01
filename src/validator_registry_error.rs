//! Validator registry errors.

use thiserror::Error;

use crate::RegistrationSource;

/// Failure while freezing a validator registry.
#[derive(Clone, Debug, Error)]
pub enum ValidatorRegistryError {
    /// Multiple registrations claim one stable ID.
    #[error("duplicate validator ID {id} from {sources:?}")]
    DuplicateId {
        /// Conflicting stable ID.
        id: &'static str,
        /// Registration sources in deterministic order.
        sources: Vec<RegistrationSource>,
    },
}
