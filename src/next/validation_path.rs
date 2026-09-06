// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Safe validation paths.

use std::borrow::Cow;

use super::PathSegment;

/// A structured path to a value being validated.
#[derive(Clone, Eq, PartialEq)]
pub struct ValidationPath {
    segments: Vec<PathSegment>,
}

impl ValidationPath {
    /// Creates an empty root path.
    #[must_use]
    pub const fn root() -> Self {
        Self { segments: Vec::new() }
    }

    /// Appends a field segment.
    #[must_use]
    pub fn with_field(mut self, field: impl Into<Cow<'static, str>>) -> Self {
        self.segments.push(PathSegment::Field(field.into()));
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
    pub fn as_segments(&self) -> &[PathSegment] {
        &self.segments
    }

    /// Explicitly renders this path for a trusted presentation layer.
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
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ValidationPath")
            .field("segment_count", &self.segments.len())
            .finish_non_exhaustive()
    }
}

impl std::fmt::Display for ValidationPath {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("<validation-path>")
    }
}
