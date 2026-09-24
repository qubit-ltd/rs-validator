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
/// Segments sort by variant in the order listed below, then by their field
/// label or numeric index within a variant.
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

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hash;
    use std::hash::Hasher;

    use super::PathSegment;

    /// Checks variant order and the values carried by field and index variants.
    #[test]
    fn test_path_segment_order() {
        let mut segments = vec![
            PathSegment::MapValue,
            PathSegment::Index(2),
            PathSegment::Field("zeta".into()),
            PathSegment::MapKey,
            PathSegment::MapEntry(3),
            PathSegment::Field("alpha".into()),
            PathSegment::Index(1),
            PathSegment::MapEntry(1),
        ];

        segments.sort();

        assert_eq!(
            segments,
            vec![
                PathSegment::Field("alpha".into()),
                PathSegment::Field("zeta".into()),
                PathSegment::Index(1),
                PathSegment::Index(2),
                PathSegment::MapEntry(1),
                PathSegment::MapEntry(3),
                PathSegment::MapKey,
                PathSegment::MapValue,
            ]
        );
    }

    /// Checks equal field labels hash equally across borrowed and owned
    /// storage.
    #[test]
    fn test_path_segment_hash_matches_equality() {
        let borrowed = PathSegment::Field("name".into());
        let owned = PathSegment::Field(String::from("name").into());
        assert_eq!(borrowed, owned);

        let hash = |segment: &PathSegment| {
            let mut hasher = DefaultHasher::new();
            segment.hash(&mut hasher);
            hasher.finish()
        };
        assert_eq!(hash(&borrowed), hash(&owned));
    }
}
