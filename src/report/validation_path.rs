// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Safe validation paths.

use super::PathSegment;

/// A structured path to a value being validated.
///
/// `Debug` and `Display` do not reveal field labels. Call [`Self::render`]
/// explicitly only in a trusted presentation layer; even rendered map entries
/// contain opaque positions rather than raw map keys.
///
/// Field labels are static program declarations. Runtime map keys must be
/// represented by opaque map-entry segments.
///
/// # Examples
///
/// ```
/// use qubit_validator::ValidationPath;
///
/// let path = ValidationPath::root().with_field("profile").with_index(2);
/// assert_eq!(path.render(), "profile[2]");
/// assert_eq!(path.as_segments().len(), 2);
/// ```
///
/// Runtime field names cannot be retained in a path. Represent dynamic map
/// entries with [`Self::with_map_entry`] instead.
///
/// ```compile_fail
/// use qubit_validator::ValidationPath;
/// let runtime_name = String::from("user supplied key");
/// let _ = ValidationPath::root().with_field(runtime_name);
/// ```
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ValidationPath {
    /// Segments stored from the root toward the validated value.
    segments: Vec<PathSegment>,
}

impl ValidationPath {
    /// Creates an empty root path.
    #[must_use]
    #[inline]
    pub const fn root() -> Self {
        Self { segments: Vec::new() }
    }

    /// Appends a field segment.
    #[must_use]
    pub fn with_field(mut self, field: &'static str) -> Self {
        self.segments.push(PathSegment::Field(field));
        self
    }

    /// Appends a sequence index segment.
    #[must_use]
    pub fn with_index(mut self, index: usize) -> Self {
        self.segments.push(PathSegment::Index(index));
        self
    }

    /// Appends an opaque map entry segment.
    #[must_use]
    pub fn with_map_entry(mut self, index: usize) -> Self {
        self.segments.push(PathSegment::MapEntry(index));
        self
    }

    /// Appends the key side of a map entry.
    #[must_use]
    pub fn with_map_key(mut self) -> Self {
        self.segments.push(PathSegment::MapKey);
        self
    }

    /// Appends the value side of a map entry.
    #[must_use]
    pub fn with_map_value(mut self) -> Self {
        self.segments.push(PathSegment::MapValue);
        self
    }

    /// Returns the path segments in order.
    #[must_use]
    #[inline]
    pub fn as_segments(&self) -> &[PathSegment] {
        &self.segments
    }

    /// Appends a relative path to this path without rendering either one.
    ///
    /// Returns a new path and leaves both inputs unchanged. Segments from
    /// `relative` follow this path's segments in their original order.
    #[must_use]
    pub fn concat(&self, relative: &Self) -> Self {
        let mut segments = Vec::with_capacity(self.segments.len() + relative.segments.len());
        segments.extend_from_slice(&self.segments);
        segments.extend_from_slice(&relative.segments);
        Self { segments }
    }

    /// Explicitly renders this path for a trusted presentation layer.
    ///
    /// The result can contain program-supplied field labels but never contains
    /// a raw validation value or map key.
    #[must_use]
    pub fn render(&self) -> String {
        let mut rendered = String::new();
        for segment in &self.segments {
            match segment {
                PathSegment::Field(field) => {
                    if !rendered.is_empty() {
                        rendered.push('.');
                    }
                    rendered.push_str(field);
                }
                PathSegment::Index(index) => {
                    rendered.push('[');
                    rendered.push_str(&index.to_string());
                    rendered.push(']');
                }
                PathSegment::MapEntry(index) => {
                    rendered.push_str(".<map-entry:");
                    rendered.push_str(&index.to_string());
                    rendered.push('>');
                }
                PathSegment::MapKey => rendered.push_str(".<map-key>"),
                PathSegment::MapValue => rendered.push_str(".<map-value>"),
            }
        }
        rendered
    }
}

impl std::fmt::Debug for ValidationPath {
    /// Formats only the segment count so field labels remain private.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ValidationPath")
            .field("segment_count", &self.segments.len())
            .finish_non_exhaustive()
    }
}

impl std::fmt::Display for ValidationPath {
    /// Formats a redacted placeholder instead of rendering path labels.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("<validation-path>")
    }
}
