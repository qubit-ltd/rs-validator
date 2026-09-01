//! Inventory registration factories.

use crate::ValidatorRegistration;

/// Factory submitted by [`register_validator!`](crate::register_validator).
#[doc(hidden)]
pub struct ValidatorRegistrationFactory(pub fn() -> ValidatorRegistration);

inventory::collect!(ValidatorRegistrationFactory);
