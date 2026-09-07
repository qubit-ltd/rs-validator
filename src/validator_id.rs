//! Stable validator identifiers.

use core::borrow::Borrow;

use crate::ValidatorIdError;

/// A validated, process-independent validator identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ValidatorId(&'static str);

impl ValidatorId {
    /// Creates a stable ID from a static string.
    ///
    /// # Panics
    ///
    /// Panics when `value` violates the point-separated ASCII protocol.
    #[must_use]
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
    pub const fn try_new(value: &'static str) -> Result<Self, ValidatorIdError> {
        match validate(value) {
            Ok(()) => Ok(Self(value)),
            Err(error) => Err(error),
        }
    }

    /// Returns the complete stable ID.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl Borrow<str> for ValidatorId {
    fn borrow(&self) -> &str {
        self.0
    }
}

/// Validates one point-separated ASCII identifier.
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
