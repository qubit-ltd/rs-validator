// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! The typed validator contract.

/// Validates a borrowed value using an immutable, typed context.
pub trait Validator<T: ?Sized, C: ?Sized = ()> {
    /// The domain error returned when the value is invalid.
    type Error: std::error::Error + 'static;

    /// Validates `value` with `context`.
    ///
    /// # Errors
    ///
    /// Returns the validator-specific error when the value is invalid.
    fn validate(&self, value: &T, context: &C) -> Result<(), Self::Error>;
}
