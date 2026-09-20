// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Segments of a safe validation path.

use std::borrow::Cow;

/// One location segment in a validation path.
#[derive(Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum PathSegment {
    /// A declared field name.
    Field(Cow<'static, str>),
    /// A sequence element index.
    Index(usize),
    /// An opaque map entry index.
    MapEntry(usize),
    /// The key side of a map entry.
    MapKey,
    /// The value side of a map entry.
    MapValue,
}

impl std::fmt::Debug for PathSegment {
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
