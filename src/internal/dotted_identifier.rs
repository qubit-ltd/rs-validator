// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Validation for the shared dotted ASCII identifier protocol.

use super::DottedIdentifierError;

/// Validates a non-empty dotted identifier with ASCII letter-led segments.
#[inline]
pub(crate) const fn validate_dotted_identifier(value: &str) -> Result<(), DottedIdentifierError> {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return Err(DottedIdentifierError::Empty);
    }
    let mut segment_start = 0;
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'.' {
            if segment_start == index {
                return Err(DottedIdentifierError::EmptySegment);
            }
            if let Err(error) = validate_segment(bytes, segment_start, index) {
                return Err(error);
            }
            segment_start = index + 1;
        }
        index += 1;
    }
    if segment_start == bytes.len() {
        return Err(DottedIdentifierError::EmptySegment);
    }
    validate_segment(bytes, segment_start, bytes.len())
}

/// Validates one non-empty segment using the protocol's ASCII character set.
#[inline]
const fn validate_segment(bytes: &[u8], start: usize, end: usize) -> Result<(), DottedIdentifierError> {
    if start == end || !bytes[start].is_ascii_alphabetic() {
        return Err(DottedIdentifierError::InvalidSegment);
    }
    let mut index = start + 1;
    while index < end {
        let byte = bytes[index];
        if !(byte.is_ascii_alphanumeric() || byte == b'_') {
            return Err(DottedIdentifierError::InvalidSegment);
        }
        index += 1;
    }
    Ok(())
}
