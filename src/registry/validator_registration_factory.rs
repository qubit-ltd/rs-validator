// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Inventory registration factories.

use crate::ValidatorRegistration;

/// Factory submitted by [`register_validator!`](crate::register_validator).
///
/// This inventory integration type is available only with the `inventory`
/// feature and is hidden from the normal public API documentation.
#[doc(hidden)]
pub struct ValidatorRegistrationFactory(
    /// Function invoked while constructing the process-wide registry.
    pub fn() -> ValidatorRegistration,
);

inventory::collect!(ValidatorRegistrationFactory);

/// Registers a static typed validator descriptor in the optional global
/// registry.
///
/// This macro is available only with the `inventory` feature. It submits a
/// compile-time factory; the registration is materialized when
/// [`ValidatorRegistry::try_global`](crate::ValidatorRegistry::try_global) is
/// first called.
#[macro_export]
macro_rules! register_validator {
    (registration = $registration:path $(,)?) => {
        const _: () = {
            fn registration() -> $crate::ValidatorRegistration {
                $registration
            }

            $crate::__private::inventory::submit! {
                $crate::ValidatorRegistrationFactory(registration)
            }
        };
    };
    (id = $id:literal, descriptor = $descriptor:expr $(,)?) => {
        const _: () = {
            fn registration() -> $crate::ValidatorRegistration {
                $crate::ValidatorRegistration::new(
                    $crate::ValidatorId::new($id),
                    $descriptor,
                    $crate::RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
                )
            }

            $crate::__private::inventory::submit! {
                $crate::ValidatorRegistrationFactory(registration)
            }
        };
    };
}
