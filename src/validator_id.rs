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
    /// # Panics
    ///
    /// Panics when `value` violates the point-separated ASCII protocol.
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
    /// # Errors
    ///
    /// Returns the exact protocol violation for an invalid ID.
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
    #[must_use]
    #[inline]
    fn borrow(&self) -> &str {
        self.0
    }
}
