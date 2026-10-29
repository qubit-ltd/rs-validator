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
/// The literal ID is validated during compilation. Invalid literal IDs are
/// rejected before a global registry can be initialized.
///
/// When a registration value is already shared by local and process-wide
/// registries, pass its path to `registration =`. This form preserves the
/// registration's existing source metadata. The `id`/`descriptor` form
/// instead records the macro invocation's crate, module, file, and line.
///
/// # Panics
///
/// The descriptor expression is evaluated when the registration is
/// materialized; panics from that expression are not caught by the registry.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use qubit_validator::ArgumentReader;
/// use qubit_validator::BindError;
/// use qubit_validator::InputType;
/// use qubit_validator::PreparedOutcome;
/// use qubit_validator::PreparedValidator;
/// use qubit_validator::ValidatorDescriptor;
/// use qubit_validator::ValidatorSignature;
/// use qubit_validator::prepare_text_with_context;
/// use qubit_validator::register_validator;
///
/// fn prepare(_: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
///     Ok(prepare_text_with_context(&[], |_, _| Ok(PreparedOutcome::Valid)))
/// }
/// static SIGNATURES: &[ValidatorSignature] =
///     &[ValidatorSignature::new(InputType::Text, &[], prepare)];
/// static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
/// register_validator!(id = "app.valid_rule", descriptor = &DESCRIPTOR);
/// ```
///
/// A shared registration can be used in both a local registry and the global
/// inventory. The registration path preserves the source stored in the value.
///
/// ```
/// use std::sync::Arc;
/// use qubit_validator::{
///     ArgumentReader, BindError, InputType, PreparedOutcome, PreparedValidator,
///     RegistrationSource, ValidatorDescriptor, ValidatorId, ValidatorRegistration,
///     ValidatorRegistry, ValidatorSignature, prepare_text_with_context,
///     register_validator,
/// };
///
/// fn prepare(_: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
///     Ok(prepare_text_with_context(&[], |_, _| Ok(PreparedOutcome::Valid)))
/// }
/// static SIGNATURES: &[ValidatorSignature] =
///     &[ValidatorSignature::new(InputType::Text, &[], prepare)];
/// static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
/// const SHARED_REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
///     ValidatorId::new("app.shared_rule"),
///     &DESCRIPTOR,
///     RegistrationSource::new("app", "app::rules", "src/rules.rs", 42),
/// );
///
/// let local = ValidatorRegistry::from_registrations([SHARED_REGISTRATION]).unwrap();
/// register_validator!(registration = SHARED_REGISTRATION);
/// let global = ValidatorRegistry::try_global().unwrap();
/// let local_entry = local.get("app.shared_rule").unwrap();
/// let global_entry = global.get("app.shared_rule").unwrap();
/// assert_eq!(local_entry.id(), global_entry.id());
/// assert!(std::ptr::eq(local_entry.descriptor(), global_entry.descriptor()));
/// assert_eq!(local_entry.source(), global_entry.source());
/// ```
///
/// ```compile_fail
/// use std::sync::Arc;
/// use qubit_validator::ArgumentReader;
/// use qubit_validator::BindError;
/// use qubit_validator::InputType;
/// use qubit_validator::PreparedOutcome;
/// use qubit_validator::PreparedValidator;
/// use qubit_validator::ValidatorDescriptor;
/// use qubit_validator::ValidatorSignature;
/// use qubit_validator::prepare_text_with_context;
/// use qubit_validator::register_validator;
///
/// fn prepare(_: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
///     Ok(prepare_text_with_context(&[], |_, _| Ok(PreparedOutcome::Valid)))
/// }
/// static SIGNATURES: &[ValidatorSignature] =
///     &[ValidatorSignature::new(InputType::Text, &[], prepare)];
/// static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
/// register_validator!(id = "bad..id", descriptor = &DESCRIPTOR);
/// ```
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
                    const { $crate::ValidatorId::new($id) },
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
