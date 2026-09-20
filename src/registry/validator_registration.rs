// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Validator registrations for the next-generation registry.

use super::ValidatorDescriptor;
use crate::RegistrationSource;
use crate::ValidatorId;

/// One validator definition associated with a stable identifier.
#[derive(Clone, Copy, Debug)]
pub struct ValidatorRegistration {
    id: ValidatorId,
    descriptor: &'static ValidatorDescriptor,
    source: RegistrationSource,
}

impl ValidatorRegistration {
    /// Creates a registration from validated static facts.
    #[doc(hidden)]
    #[must_use]
    pub const fn new(id: ValidatorId, descriptor: &'static ValidatorDescriptor, source: RegistrationSource) -> Self {
        Self { id, descriptor, source }
    }

    /// Returns the stable identifier.
    #[must_use]
    pub const fn id(&self) -> ValidatorId {
        self.id
    }

    /// Returns the immutable descriptor.
    #[must_use]
    pub const fn descriptor(&self) -> &'static ValidatorDescriptor {
        self.descriptor
    }

    /// Returns the source location used for duplicate diagnostics.
    #[must_use]
    pub const fn source(&self) -> RegistrationSource {
        self.source
    }
}
