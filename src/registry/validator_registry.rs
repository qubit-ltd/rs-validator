// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable local registries for multi-signature validator definitions.

#[cfg(feature = "inventory")]
use std::sync::OnceLock;

#[cfg(feature = "inventory")]
use inventory::iter;

use super::ValidatorRegistration;
use crate::NamedValidationArgument;
use crate::ValidatorRegistryError;
use crate::binding::BindError;
use crate::binding::BindErrorKind;
use crate::binding::BoundValidator;
use crate::binding::InputType;

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
}

impl ValidatorRegistry {
    /// Builds a registry by taking ownership of registration values.
    ///
    /// # Type Parameters
    ///
    /// * `I` - An iterable whose items can be converted into registrations.
    ///
    /// # Parameters
    ///
    /// * `registrations` - Values to validate and store in the registry.
    ///
    /// # Errors
    ///
    /// Returns every source which declared a duplicated stable ID, or the
    /// identifier and cause of the first invalid descriptor.
    ///
    /// # Returns
    ///
    /// Returns a registry containing the validated registrations sorted by ID.
    pub fn from_registrations<I>(registrations: I) -> Result<Self, ValidatorRegistryError>
    where
        I: IntoIterator,
        I::Item: Into<ValidatorRegistration>,
    {
        Self::build(registrations.into_iter().map(Into::into).collect())
    }

    /// Returns an empty registry.
    ///
    /// # Returns
    ///
    /// Returns a registry with no registrations.
    #[must_use]
    #[inline]
    pub fn empty() -> Self {
        Self {
            registrations: Box::new([]),
        }
    }

    /// Initializes and returns the process-wide inventory registry.
    ///
    /// # Errors
    ///
    /// Returns the cached error when linked registrations contain a duplicate
    /// ID or an invalid descriptor. This method is available only with the
    /// `inventory` feature.
    ///
    /// # Returns
    ///
    /// Returns the process-wide registry when initialization succeeds.
    #[cfg(feature = "inventory")]
    pub fn try_global() -> Result<&'static Self, ValidatorRegistryError> {
        static REGISTRY: OnceLock<Result<ValidatorRegistry, ValidatorRegistryError>> = OnceLock::new();
        match REGISTRY.get_or_init(|| {
            let registrations = iter::<crate::ValidatorRegistrationFactory>
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
    ///
    /// # Returns
    ///
    /// Returns the initialized process-wide registry.
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
        self.registrations
            .binary_search_by(|registration| registration.id().as_str().cmp(id))
            .ok()
            .and_then(|index| self.registrations.get(index))
    }

    /// Returns registrations sorted by stable ID.
    ///
    /// # Returns
    ///
    /// Returns a borrowed slice of all registrations in stable-ID order.
    #[must_use]
    #[inline]
    pub fn registrations(&self) -> &[ValidatorRegistration] {
        &self.registrations
    }

    /// Binds a rule by ID and input shape.
    ///
    /// # Parameters
    ///
    /// * `id` - Stable identifier of the registered rule to bind.
    /// * `input` - Input shape that the bound rule must accept.
    /// * `params` - Named arguments supplied to the rule.
    ///
    /// # Errors
    ///
    /// Returns `MissingRule` when `id` has no registration. This error has
    /// no resolved rule ID; callers that need the requested text retain `id`.
    /// Signature and preparation errors for a found rule carry its ID.
    ///
    /// # Returns
    ///
    /// Returns the bound validator when the registration exists and accepts
    /// the supplied input and parameters.
    pub fn bind(
        &self,
        id: &str,
        input: InputType,
        params: &[NamedValidationArgument<'_>],
    ) -> Result<BoundValidator, BindError> {
        let registration = self.get(id).ok_or_else(|| BindError::new(BindErrorKind::MissingRule))?;
        registration
            .descriptor()
            .bind_for_validated(registration.id(), input, params)
            .map_err(|error| error.with_rule(registration.id()))
    }

    /// Sorts and validates owned registrations before freezing the registry.
    ///
    /// # Parameters
    ///
    /// * `registrations` - Owned registrations to sort and validate.
    ///
    /// # Errors
    ///
    /// Returns duplicate-ID metadata or the first invalid descriptor.
    ///
    /// # Returns
    ///
    /// Returns a registry containing the validated registrations sorted by ID.
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
        Ok(Self {
            registrations: registrations.into_boxed_slice(),
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
