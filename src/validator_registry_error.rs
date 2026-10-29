// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Validator registry errors.

use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

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
///     dependency: None,
/// };
/// assert!(error.to_string().contains("invalid descriptor"));
/// ```
///
/// # Source compatibility
///
/// Constructing `InvalidDescriptor` directly requires the `dependency` field;
/// use `None` when no declaration dependency name is available. Complete
/// patterns must bind `dependency` or use `..` to ignore it.
#[must_use]
#[derive(Clone, Error)]
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
        /// Dependency name attached to the declaration error, if present.
        ///
        /// This stores only names declared in the descriptor, not runtime
        /// dependency values. The name is available through pattern matching
        /// but is omitted from the default debug representation.
        dependency: Option<String>,
    },
}

impl Debug for ValidatorRegistryError {
    /// Formats registration metadata without exposing dependency names.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::DuplicateId { id, sources } => formatter
                .debug_struct("DuplicateId")
                .field("id", id)
                .field("sources", sources)
                .finish(),
            Self::InvalidDescriptor {
                id,
                registration_source,
                kind,
                dependency,
            } => formatter
                .debug_struct("InvalidDescriptor")
                .field("id", id)
                .field("registration_source", registration_source)
                .field("kind", kind)
                .field("has_dependency", &dependency.is_some())
                .finish(),
        }
    }
}
