// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Stable validation violation codes.

use super::ViolationCodeError;

/// A stable violation code using the point-separated ASCII protocol.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ViolationCode(&'static str);

impl ViolationCode {
    /// Creates a code from a valid static string.
    ///
    /// # Panics
    ///
    /// Panics when `value` violates the code protocol.
    #[must_use]
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
    pub const fn try_new(
        value: &'static str,
    ) -> Result<Self, ViolationCodeError> {
        let bytes = value.as_bytes();
        if bytes.is_empty() {
            return Err(ViolationCodeError::Empty);
        }

        let mut start = 0;
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'.' {
                if start == index {
                    return Err(ViolationCodeError::EmptySegment);
                }
                if !valid_segment(bytes, start, index) {
                    return Err(ViolationCodeError::InvalidSegment);
                }
                start = index + 1;
            }
            index += 1;
        }
        if start == bytes.len() {
            return Err(ViolationCodeError::EmptySegment);
        }
        if !valid_segment(bytes, start, bytes.len()) {
            return Err(ViolationCodeError::InvalidSegment);
        }
        Ok(Self(value))
    }

    /// Returns the complete stable code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl std::fmt::Debug for ViolationCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("ViolationCode")
            .field(&self.0)
            .finish()
    }
}

impl std::fmt::Display for ViolationCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.0)
    }
}

const fn valid_segment(bytes: &[u8], start: usize, end: usize) -> bool {
    if start == end || !bytes[start].is_ascii_alphabetic() {
        return false;
    }
    let mut index = start + 1;
    while index < end {
        let byte = bytes[index];
        if !(byte.is_ascii_alphanumeric() || byte == b'_') {
            return false;
        }
        index += 1;
    }
    true
}
