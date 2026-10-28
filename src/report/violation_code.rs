// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Stable validation violation codes.

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use super::ViolationCodeError;
use crate::internal::DottedIdentifierError;
use crate::internal::validate_dotted_identifier;

/// A stable violation code using the point-separated ASCII protocol.
///
/// The value is non-empty and consists of one or more dot-separated segments.
/// Each segment starts with an ASCII letter and continues with ASCII letters,
/// digits, or underscores. Empty segments, whitespace, non-ASCII letters, and
/// hyphens are rejected. No trimming or case normalization is performed.
///
/// # Examples
///
/// ```
/// use qubit_validator::ViolationCode;
/// use qubit_validator::ViolationCodeError;
///
/// let code = ViolationCode::try_new("text.length.minimum")?;
/// assert_eq!(code.as_str(), "text.length.minimum");
/// let error = ViolationCode::try_new("text..minimum").unwrap_err();
/// assert_eq!(error, ViolationCodeError::EmptySegment);
/// assert_eq!(
///     ViolationCode::try_new("text.min-length"),
///     Err(ViolationCodeError::InvalidSegment),
/// );
/// # Ok::<(), ViolationCodeError>(())
/// ```
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ViolationCode(
    /// Validated static protocol string.
    &'static str,
);

impl ViolationCode {
    /// Creates a code from a valid static string.
    ///
    /// # Parameters
    ///
    /// - `value`: A static string containing the complete protocol value.
    ///
    /// # Returns
    ///
    /// The validated violation code.
    ///
    /// # Panics
    ///
    /// Panics when `value` violates the syntax documented on [`ViolationCode`].
    #[must_use]
    #[inline]
    pub const fn new(value: &'static str) -> Self {
        match Self::try_new(value) {
            Ok(code) => code,
            Err(_) => panic!("invalid violation code"),
        }
    }

    /// Validates and creates a code.
    ///
    /// # Parameters
    ///
    /// - `value`: A static string containing the complete protocol value.
    ///
    /// # Returns
    ///
    /// The validated violation code when `value` follows the protocol.
    ///
    /// # Errors
    ///
    /// Returns [`ViolationCodeError::Empty`] for an empty string,
    /// [`ViolationCodeError::EmptySegment`] for an empty dot-separated segment,
    /// or [`ViolationCodeError::InvalidSegment`] for a segment with invalid
    /// characters or an invalid first character.
    #[inline]
    pub const fn try_new(value: &'static str) -> Result<Self, ViolationCodeError> {
        match validate_dotted_identifier(value) {
            Ok(()) => Ok(Self(value)),
            Err(DottedIdentifierError::Empty) => Err(ViolationCodeError::Empty),
            Err(DottedIdentifierError::EmptySegment) => Err(ViolationCodeError::EmptySegment),
            Err(DottedIdentifierError::InvalidSegment) => Err(ViolationCodeError::InvalidSegment),
        }
    }

    /// Returns the complete stable code.
    #[must_use]
    #[inline]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl Debug for ViolationCode {
    /// Formats the validated, program-declared code.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.debug_tuple("ViolationCode").field(&self.0).finish()
    }
}

impl Display for ViolationCode {
    /// Formats the validated, program-declared code.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str(self.0)
    }
}
