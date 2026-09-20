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
        match validate(value) {
            Ok(()) => Ok(Self(value)),
            Err(error) => Err(error),
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

/// Validates one point-separated ASCII identifier.
#[inline]
const fn validate(value: &str) -> Result<(), ValidatorIdError> {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return Err(ValidatorIdError::Empty);
    }
    let mut segment_start = 0;
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'.' {
            if segment_start == index || index + 1 == bytes.len() {
                return Err(ValidatorIdError::EmptySegment);
            }
            if let Err(error) = validate_segment(bytes, segment_start, index) {
                return Err(error);
            }
            segment_start = index + 1;
        }
        index += 1;
    }
    validate_segment(bytes, segment_start, bytes.len())
}

/// Validates one non-empty identifier segment.
#[inline]
const fn validate_segment(bytes: &[u8], start: usize, end: usize) -> Result<(), ValidatorIdError> {
    if start == end || !bytes[start].is_ascii_alphabetic() {
        return Err(ValidatorIdError::InvalidSegment);
    }
    let mut index = start + 1;
    while index < end {
        let byte = bytes[index];
        if !(byte.is_ascii_alphanumeric() || byte == b'_') {
            return Err(ValidatorIdError::InvalidSegment);
        }
        index += 1;
    }
    Ok(())
}
