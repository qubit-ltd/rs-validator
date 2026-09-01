// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Typed validator contract.

use crate::ValidationContext;

/// Validates values of `T` with immutable call-site context.
pub trait Validator<T: ?Sized> {
    /// Domain error returned when validation rejects a value.
    type Error: std::error::Error + 'static;

    /// Validates `value` using the supplied parameters and dependencies.
    ///
    /// # Errors
    ///
    /// Returns the validator-specific error when `value` is invalid.
    fn validate(&mut self, value: &T, context: &ValidationContext<'_>) -> Result<(), Self::Error>;
}
