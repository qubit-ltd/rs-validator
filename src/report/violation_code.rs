// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Stable validation violation codes.

use super::ViolationCodeError;
use crate::internal::DottedIdentifierError;
use crate::internal::validate_dotted_identifier;

/// A stable violation code using the point-separated ASCII protocol.
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
    /// # Panics
    ///
    /// Panics when `value` violates the code protocol.
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
    /// # Errors
    ///
    /// Returns the protocol violation when `value` is invalid.
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

impl std::fmt::Debug for ViolationCode {
    /// Formats the validated, program-declared code.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_tuple("ViolationCode").field(&self.0).finish()
    }
}

impl std::fmt::Display for ViolationCode {
    /// Formats the validated, program-declared code.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.0)
    }
}
