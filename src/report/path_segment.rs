// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Segments of a safe validation path.

use std::borrow::Cow;

/// One location segment in a validation path.
///
/// # Examples
///
/// ```
/// use qubit_validator::PathSegment;
///
/// let field = PathSegment::Field("profile".into());
/// assert_eq!(format!("{field:?}"), "Field(<redacted>)");
/// ```
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum PathSegment {
    /// A declared field name.
    Field(
        /// Program-supplied field label; debug formatting redacts it.
        Cow<'static, str>,
    ),
    /// A sequence element index.
    Index(
        /// Zero-based sequence index.
        usize,
    ),
    /// An opaque map entry index.
    MapEntry(
        /// Opaque map-entry position, never the raw map key.
        usize,
    ),
    /// The key side of a map entry.
    MapKey,
    /// The value side of a map entry.
    MapValue,
}

impl std::fmt::Debug for PathSegment {
    /// Formats structural path data while redacting field labels.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field(_) => formatter.write_str("Field(<redacted>)"),
            Self::Index(index) => formatter.debug_tuple("Index").field(index).finish(),
            Self::MapEntry(index) => formatter.debug_tuple("MapEntry").field(index).finish(),
            Self::MapKey => formatter.write_str("MapKey"),
            Self::MapValue => formatter.write_str("MapValue"),
        }
    }
}
