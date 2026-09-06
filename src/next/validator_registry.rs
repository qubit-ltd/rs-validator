//! Immutable local registries for multi-signature validator definitions.

use std::collections::BTreeMap;
#[cfg(feature = "inventory")]
use std::sync::OnceLock;

use crate::NamedValidationArgument;
use crate::ValidatorId;
use crate::ValidatorRegistryError;

use super::BindError;
use super::BindErrorKind;
use super::BoundValidator;
use super::InputType;
use super::ValidatorRegistration;

/// A deterministic local registry containing one definition per stable ID.
#[derive(Debug)]
pub struct ValidatorRegistry {
    registrations: Box<[ValidatorRegistration]>,
    indices: BTreeMap<ValidatorId, usize>,
}

impl ValidatorRegistry {
    /// Builds a registry by taking ownership of registration values.
    ///
    /// # Errors
    ///
    /// Returns every source which declared a duplicated stable ID.
    pub fn from_registrations(
        registrations: impl IntoIterator<Item = ValidatorRegistration>,
    ) -> Result<Self, ValidatorRegistryError> {
        Self::build(registrations.into_iter().collect())
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
    /// Returns the cached duplicate-ID error when linked registrations
    /// conflict. This method is available only with the `inventory` feature.
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
    /// Panics when linked registrations contain duplicate IDs. This method is
    /// available only with the `inventory` feature.
    #[cfg(feature = "inventory")]
    #[must_use]
    pub fn global() -> &'static Self {
        Self::try_global().unwrap_or_else(|error| panic!("invalid global validator registry: {error}"))
    }

    /// Finds a registration by stable ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&ValidatorRegistration> {
        self.indices
            .get(id)
            .and_then(|index| self.registrations.get(*index))
    }

    /// Returns registrations sorted by stable ID.
    #[must_use]
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
        let registration = self
            .get(id)
            .ok_or_else(|| BindError::new(BindErrorKind::MissingRule))?;
        registration
            .descriptor()
            .bind_for(input, params)
            .map(|bound| bound.with_rule(registration.id()))
            .map_err(|error| error.with_rule(registration.id()))
    }

    fn build(
        mut registrations: Vec<ValidatorRegistration>,
    ) -> Result<Self, ValidatorRegistryError> {
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
impl From<&'static ValidatorRegistration> for ValidatorRegistration {
    fn from(value: &'static ValidatorRegistration) -> Self {
        *value
    }
}
