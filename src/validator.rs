// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! The typed validator contract.

/// Validates a borrowed value using an immutable, typed context.
///
/// # Examples
///
/// ```
/// use std::fmt;
///
/// use qubit_validator::Validator;
///
/// #[derive(Debug, Eq, PartialEq)]
/// struct EmptyText;
///
/// impl fmt::Display for EmptyText {
///     fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
///         formatter.write_str("text must not be empty")
///     }
/// }
///
/// impl std::error::Error for EmptyText {}
///
/// struct NonEmpty;
///
/// impl Validator<str> for NonEmpty {
///     type Error = EmptyText;
///
///     fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
///         if value.is_empty() { Err(EmptyText) } else { Ok(()) }
///     }
/// }
///
/// assert!(NonEmpty.validate("ready", &()).is_ok());
/// assert_eq!(NonEmpty.validate("", &()).unwrap_err(), EmptyText);
/// ```
///
/// # Type Parameters
///
/// - `T`: Borrowed value type accepted by the validator.
/// - `C`: Immutable context supplied for each validation call.
pub trait Validator<T: ?Sized, C: ?Sized = ()> {
    /// The domain error returned when the value is invalid.
    type Error: std::error::Error + 'static;

    /// Validates `value` with `context`.
    ///
    /// # Parameters
    ///
    /// - `value`: The borrowed value to validate.
    /// - `context`: Immutable domain context for this call.
    ///
    /// # Errors
    ///
    /// Returns the validator-specific error when the value is invalid.
    fn validate(&self, value: &T, context: &C) -> Result<(), Self::Error>;
}
