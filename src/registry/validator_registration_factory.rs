//! Inventory registration factories.

use crate::ValidatorRegistration;

/// Factory submitted by [`register_validator!`](crate::register_validator).
#[doc(hidden)]
pub struct ValidatorRegistrationFactory(pub fn() -> ValidatorRegistration);

inventory::collect!(ValidatorRegistrationFactory);

/// Registers a static typed validator descriptor in the optional global
/// registry.
#[macro_export]
macro_rules! register_validator {
    (id = $id:literal, descriptor = $descriptor:expr $(,)?) => {
        const _: () = {
            fn registration() -> $crate::ValidatorRegistration {
                $crate::ValidatorRegistration::new(
                    $crate::ValidatorId::new($id),
                    $descriptor,
                    $crate::RegistrationSource::new(
                        env!("CARGO_PKG_NAME"),
                        module_path!(),
                        file!(),
                        line!(),
                    ),
                )
            }

            $crate::__private::inventory::submit! {
                $crate::ValidatorRegistrationFactory(registration)
            }
        };
    };
}
