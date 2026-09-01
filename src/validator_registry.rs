//! Immutable local and process-wide validator registries.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::ValidatorId;
use crate::ValidatorRegistration;
use crate::ValidatorRegistrationFactory;
use crate::ValidatorRegistryError;

/// An immutable validator registry sorted by stable ID.
#[derive(Debug)]
pub struct ValidatorRegistry {
    registrations: Box<[ValidatorRegistration]>,
    indices: BTreeMap<ValidatorId, usize>,
}

impl ValidatorRegistry {
    /// Builds a local registry from static registrations.
    ///
    /// # Errors
    ///
    /// Returns a duplicate-ID error when multiple entries claim one ID.
    pub fn from_registrations(
        registrations: impl IntoIterator<Item = &'static ValidatorRegistration>,
    ) -> Result<Self, ValidatorRegistryError> {
        Self::build(registrations.into_iter().copied().collect())
    }

    /// Returns an empty local registry.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            registrations: Box::new([]),
            indices: BTreeMap::new(),
        }
    }

    /// Initializes and returns the process-wide linked registry.
    ///
    /// # Errors
    ///
    /// Returns the cached construction error when linked registrations
    /// conflict.
    pub fn try_global() -> Result<&'static Self, ValidatorRegistryError> {
        static REGISTRY: OnceLock<Result<ValidatorRegistry, ValidatorRegistryError>> = OnceLock::new();
        match REGISTRY.get_or_init(|| {
            let registrations = inventory::iter::<ValidatorRegistrationFactory>
                .into_iter()
                .map(|factory| (factory.0)())
                .collect();
            Self::build(registrations)
        }) {
            Ok(registry) => Ok(registry),
            Err(error) => Err(error.clone()),
        }
    }

    /// Returns the process-wide registry or panics with a stable diagnostic.
    ///
    /// # Panics
    ///
    /// Panics when linked registrations conflict.
    #[must_use]
    pub fn global() -> &'static Self {
        Self::try_global().unwrap_or_else(|error| panic!("invalid global validator registry: {error}"))
    }

    /// Finds a registration by stable ID.
    ///
    /// Returns `None` when `id` is absent.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&ValidatorRegistration> {
        let index = *self.indices.get(id)?;
        self.registrations.get(index)
    }

    /// Returns registrations in deterministic ID order.
    #[must_use]
    pub fn registrations(&self) -> &[ValidatorRegistration] {
        &self.registrations
    }

    /// Freezes registrations and rejects duplicate IDs.
    fn build(mut registrations: Vec<ValidatorRegistration>) -> Result<Self, ValidatorRegistryError> {
        registrations.sort_by_key(ValidatorRegistration::id);
        for pair in registrations.windows(2) {
            if pair[0].id() == pair[1].id() {
                let mut sources = pair.iter().map(ValidatorRegistration::source).collect::<Vec<_>>();
                sources.sort_unstable();
                return Err(ValidatorRegistryError::DuplicateId {
                    id: pair[0].id().as_str(),
                    sources,
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
