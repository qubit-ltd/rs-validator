// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Validator registry errors.

use thiserror::Error;

use crate::BindErrorKind;
use crate::RegistrationSource;
use crate::ValidatorId;

/// Failure while freezing a validator registry.
///
/// # Examples
///
/// ```
/// use qubit_validator::{BindErrorKind, RegistrationSource, ValidatorId, ValidatorRegistryError};
///
/// let error = ValidatorRegistryError::InvalidDescriptor {
///     id: ValidatorId::new("text.required"),
///     registration_source: RegistrationSource::new("rules", "rules::text", "src/text.rs", 12),
///     kind: BindErrorKind::InvalidDeclaration,
/// };
/// assert!(error.to_string().contains("invalid descriptor"));
/// ```
#[must_use]
#[derive(Clone, Debug, Error)]
#[non_exhaustive]
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
