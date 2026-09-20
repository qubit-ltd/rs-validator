//! Validator registry errors.

use thiserror::Error;

use crate::BindErrorKind;
use crate::RegistrationSource;
use crate::ValidatorId;

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
    /// A registration contains an invalid descriptor declaration.
    #[error("invalid descriptor for validator ID {id:?} from {registration_source:?}: {kind}")]
    InvalidDescriptor {
        /// Validator identifier.
        id: ValidatorId,
        /// Registration source.
        registration_source: RegistrationSource,
        /// Declaration error kind.
        kind: BindErrorKind,
    },
}
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
