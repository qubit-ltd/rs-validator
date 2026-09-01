//! Distributed validator registrations.

use crate::RegistrationSource;
use crate::ValidatorDescriptor;
use crate::ValidatorId;

/// One statically linked validator implementation.
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

    /// Returns the stable validator ID.
    #[must_use]
    pub const fn id(&self) -> ValidatorId {
        self.id
    }

    /// Returns the executable descriptor.
    #[must_use]
    pub const fn descriptor(&self) -> &'static ValidatorDescriptor {
        self.descriptor
    }

    /// Returns the linked source location.
    #[must_use]
    pub const fn source(&self) -> RegistrationSource {
        self.source
    }
}

/// Registers a default-constructible validator under a stable ID.
#[macro_export]
macro_rules! register_validator {
    (id = $id:literal, validator = $validator:ty, value = $value:ty $(,)?) => {
        const _: () = {
            static DESCRIPTOR: $crate::ValidatorDescriptor = $crate::ValidatorDescriptor::of::<$validator, $value>();

            fn registration() -> $crate::ValidatorRegistration {
                $crate::ValidatorRegistration::new(
                    $crate::ValidatorId::new($id),
                    &DESCRIPTOR,
                    $crate::RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
                )
            }

            $crate::__private::inventory::submit! {
                $crate::ValidatorRegistrationFactory(registration)
            }
        };
    };
}
