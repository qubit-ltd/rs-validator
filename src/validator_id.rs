// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Stable validator identifiers.

use core::borrow::Borrow;

use crate::ValidatorIdError;
use crate::internal::DottedIdentifierError;
use crate::internal::validate_dotted_identifier;

/// A validated, process-independent validator identifier.
///
/// The value is non-empty and consists of one or more dot-separated segments.
/// Each segment starts with an ASCII letter and continues with ASCII letters,
/// digits, or underscores. Empty segments, whitespace, non-ASCII letters, and
/// hyphens are rejected. No trimming or case normalization is performed.
///
/// # Examples
///
/// ```
/// use qubit_validator::ValidatorId;
/// use qubit_validator::ValidatorIdError;
///
/// let id = ValidatorId::try_new("qubit.text.non_empty")?;
/// assert_eq!(id.as_str(), "qubit.text.non_empty");
/// let error = ValidatorId::try_new("qubit..non_empty").unwrap_err();
/// assert_eq!(error, ValidatorIdError::EmptySegment);
/// assert_eq!(
///     ValidatorId::try_new("_custom.invalid"),
///     Err(ValidatorIdError::InvalidSegment),
/// );
/// # Ok::<(), ValidatorIdError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ValidatorId(
    /// Validated static protocol string.
    &'static str,
);

impl ValidatorId {
    /// Creates a stable ID from a static string.
    ///
    /// # Parameters
    ///
    /// - `value`: A static string containing the complete protocol value.
    ///
    /// # Returns
    ///
    /// The validated identifier.
    ///
    /// # Panics
    ///
    /// Panics when `value` violates the syntax documented on [`ValidatorId`].
    #[must_use]
    #[inline]
    pub const fn new(value: &'static str) -> Self {
        match Self::try_new(value) {
            Ok(id) => id,
            Err(_) => panic!("invalid validator ID"),
        }
    }

    /// Validates and creates a stable ID.
    ///
    /// # Parameters
    ///
    /// - `value`: A static string containing the complete protocol value.
    ///
    /// # Returns
    ///
    /// The validated identifier when `value` follows the protocol.
    ///
    /// # Errors
    ///
    /// Returns [`ValidatorIdError::Empty`] for an empty string,
    /// [`ValidatorIdError::EmptySegment`] for an empty dot-separated segment,
    /// or [`ValidatorIdError::InvalidSegment`] for a segment with invalid
    /// characters or an invalid first character.
    #[inline]
    pub const fn try_new(value: &'static str) -> Result<Self, ValidatorIdError> {
        match validate_dotted_identifier(value) {
            Ok(()) => Ok(Self(value)),
            Err(DottedIdentifierError::Empty) => Err(ValidatorIdError::Empty),
            Err(DottedIdentifierError::EmptySegment) => Err(ValidatorIdError::EmptySegment),
            Err(DottedIdentifierError::InvalidSegment) => Err(ValidatorIdError::InvalidSegment),
        }
    }

    /// Returns the complete stable ID.
    #[must_use]
    #[inline]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl Borrow<str> for ValidatorId {
    /// Borrows the complete validated identifier as text.
    #[inline]
    fn borrow(&self) -> &str {
        self.0
    }
}
