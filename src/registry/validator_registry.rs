// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable local registries for multi-signature validator definitions.

use std::collections::BTreeMap;
#[cfg(feature = "inventory")]
use std::sync::OnceLock;

use super::BindError;
use super::BindErrorKind;
use super::BoundValidator;
use super::InputType;
use super::ValidatorRegistration;
use crate::NamedValidationArgument;
use crate::ValidatorId;
use crate::ValidatorRegistryError;

/// A deterministic local registry containing one definition per stable ID.
///
/// # Examples
///
/// ```
/// use qubit_validator::ValidatorRegistry;
///
/// let registry = ValidatorRegistry::empty();
/// assert!(registry.registrations().is_empty());
/// assert!(registry.get("example.unknown").is_none());
/// ```
#[derive(Debug)]
pub struct ValidatorRegistry {
    /// Registrations sorted by stable identifier.
    registrations: Box<[ValidatorRegistration]>,
    /// Lookup table mapping each stable identifier to its sorted position.
    indices: BTreeMap<ValidatorId, usize>,
}

impl ValidatorRegistry {
    /// Builds a registry by taking ownership of registration values.
    ///
    /// # Errors
    ///
    /// Returns every source which declared a duplicated stable ID, or the
    /// identifier and cause of the first invalid descriptor.
    pub fn from_registrations<I>(registrations: I) -> Result<Self, ValidatorRegistryError>
    where
        I: IntoIterator,
        I::Item: Into<ValidatorRegistration>,
    {
        Self::build(registrations.into_iter().map(Into::into).collect())
    }

    /// Returns an empty registry.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            registrations: Box::new([]),
            indices: BTreeMap::new(),
        }
    }

    /// Initializes and returns the process-wide inventory registry.
    ///
    /// # Errors
    ///
    /// Returns the cached error when linked registrations contain a duplicate
    /// ID or an invalid descriptor. This method is available only with the
    /// `inventory` feature.
    #[cfg(feature = "inventory")]
    pub fn try_global() -> Result<&'static Self, ValidatorRegistryError> {
        static REGISTRY: OnceLock<Result<ValidatorRegistry, ValidatorRegistryError>> = OnceLock::new();
        match REGISTRY.get_or_init(|| {
            let registrations = inventory::iter::<crate::ValidatorRegistrationFactory>
                .into_iter()
                .map(|factory| (factory.0)())
                .collect();
            Self::build(registrations)
        }) {
            Ok(registry) => Ok(registry),
            Err(error) => Err(error.clone()),
        }
    }

    /// Returns the process-wide inventory registry.
    ///
    /// # Panics
    ///
    /// Panics when linked registrations contain duplicate IDs or an invalid
    /// descriptor. This method is available only with the `inventory` feature.
    #[cfg(feature = "inventory")]
    #[must_use]
    pub fn global() -> &'static Self {
        Self::try_global().unwrap_or_else(|error| panic!("invalid global validator registry: {error}"))
    }

    /// Finds a registration by stable ID.
    ///
    /// # Returns
    ///
    /// Returns `Some` for a registered ID and `None` when the ID is unknown.
    #[must_use]
    #[inline]
    pub fn get(&self, id: &str) -> Option<&ValidatorRegistration> {
        self.indices.get(id).and_then(|index| self.registrations.get(*index))
    }

    /// Returns registrations sorted by stable ID.
    #[must_use]
    #[inline]
    pub fn registrations(&self) -> &[ValidatorRegistration] {
        &self.registrations
    }

    /// Binds a rule by ID and input shape.
    ///
    /// # Errors
    ///
    /// Returns a missing-rule, signature, or parameter binding error.
    pub fn bind(
        &self,
        id: &str,
        input: InputType,
        params: &[NamedValidationArgument<'_>],
    ) -> Result<BoundValidator, BindError> {
        let registration = self.get(id).ok_or_else(|| BindError::new(BindErrorKind::MissingRule))?;
        registration
            .descriptor()
            .bind_for(registration.id(), input, params)
            .map_err(|error| error.with_rule(registration.id()))
    }

    /// Sorts and validates owned registrations before building the lookup
    /// table.
    ///
    /// # Errors
    ///
    /// Returns duplicate-ID metadata or the first invalid descriptor.
    fn build(mut registrations: Vec<ValidatorRegistration>) -> Result<Self, ValidatorRegistryError> {
        registrations.sort_by_key(|registration| (registration.id(), registration.source()));
        let mut index = 0;
        while index < registrations.len() {
            let id = registrations[index].id();
            let end = index
                + registrations[index..]
                    .iter()
                    .take_while(|registration| registration.id() == id)
                    .count();
            if end - index > 1 {
                let sources = registrations[index..end]
                    .iter()
                    .map(ValidatorRegistration::source)
                    .collect();
                return Err(ValidatorRegistryError::DuplicateId {
                    id: id.as_str(),
                    sources,
                });
            }
            index = end;
        }
        for registration in &registrations {
            if let Err(error) = registration.descriptor().validate_definition() {
                return Err(ValidatorRegistryError::InvalidDescriptor {
                    id: registration.id(),
                    registration_source: registration.source(),
                    kind: error.kind(),
                });
            }
        }
        let indices = registrations
            .iter()
            .enumerate()
            .map(|(index, registration)| (registration.id(), index))
            .collect();
        Ok(Self {
            registrations: registrations.into_boxed_slice(),
            indices,
        })
    }
}

// Compatibility helper for callers which still hold static references.
impl From<&ValidatorRegistration> for ValidatorRegistration {
    /// Copies a static-reference-compatible registration into the owned
    /// registry input.
    #[inline]
    fn from(value: &ValidatorRegistration) -> Self {
        *value
    }
}
