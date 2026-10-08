// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Safe validation paths.

use super::PathSegment;

fn is_simple_field(field: &str) -> bool {
    let mut bytes = field.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == b'_') && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn append_quoted_field(output: &mut String, field: &str) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    output.push_str("[\"");
    for character in field.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{0008}' => output.push_str("\\b"),
            '\u{000c}' => output.push_str("\\f"),
            control if (control as u32) < 0x20 => {
                let byte = control as u8;
                output.push_str("\\u00");
                output.push(HEX[(byte >> 4) as usize] as char);
                output.push(HEX[(byte & 0x0f) as usize] as char);
            }
            other => output.push(other),
        }
    }
    output.push_str("\"]");
}

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
    ///
    /// # Returns
    ///
    /// A path with no segments. Rendering this path produces an empty string.
    #[must_use]
    #[inline]
    pub const fn root() -> Self {
        Self { segments: Vec::new() }
    }

    /// Appends a statically declared field segment.
    ///
    /// Use only program-declared field labels. Runtime map keys belong in an
    /// opaque map-entry segment so their text is not retained.
    ///
    /// # Parameters
    ///
    /// - `field`: Static label of the declared field.
    ///
    /// # Returns
    ///
    /// The path with `field` appended after its existing segments.
    #[must_use]
    pub fn with_field(mut self, field: &'static str) -> Self {
        self.segments.push(PathSegment::Field(field));
        self
    }

    /// Appends a zero-based sequence index segment.
    ///
    /// # Parameters
    ///
    /// - `index`: Position of the element in its sequence.
    ///
    /// # Returns
    ///
    /// The path with the index appended after its existing segments.
    #[must_use]
    pub fn with_index(mut self, index: usize) -> Self {
        self.segments.push(PathSegment::Index(index));
        self
    }

    /// Appends an opaque map entry position without retaining its key.
    ///
    /// # Parameters
    ///
    /// - `index`: Stable position assigned by the caller to the map entry.
    ///
    /// # Returns
    ///
    /// The path with the map-entry position appended.
    #[must_use]
    pub fn with_map_entry(mut self, index: usize) -> Self {
        self.segments.push(PathSegment::MapEntry(index));
        self
    }

    /// Appends the key side of a map entry without storing the key itself.
    ///
    /// # Returns
    ///
    /// The path with a map-key marker appended.
    #[must_use]
    pub fn with_map_key(mut self) -> Self {
        self.segments.push(PathSegment::MapKey);
        self
    }

    /// Appends the value side of a map entry.
    ///
    /// # Returns
    ///
    /// The path with a map-value marker appended.
    #[must_use]
    pub fn with_map_value(mut self) -> Self {
        self.segments.push(PathSegment::MapValue);
        self
    }

    /// Returns the path segments in order.
    ///
    /// # Returns
    ///
    /// A borrowed slice ordered from the root toward the target.
    #[must_use]
    #[inline]
    pub fn as_segments(&self) -> &[PathSegment] {
        &self.segments
    }

    /// Appends a relative path to this path without rendering either one.
    ///
    /// Returns a new path and leaves both inputs unchanged. Segments from
    /// `relative` follow this path's segments in their original order.
    ///
    /// # Parameters
    ///
    /// - `relative`: Relative suffix to append to this path.
    ///
    /// # Returns
    ///
    /// A newly allocated path containing both segment sequences.
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
    /// a raw validation value or map key. Simple ASCII identifiers use dotted
    /// notation; other field labels use JSON-escaped `[...]` notation. This
    /// changes the rendered format for special field labels.
    ///
    /// # Returns
    ///
    /// A newly allocated string containing the rendered field and positional
    /// segments, or an empty string for the root path.
    #[must_use]
    pub fn render(&self) -> String {
        let mut rendered = String::new();
        for segment in &self.segments {
            match segment {
                PathSegment::Field(field) => {
                    if !rendered.is_empty() {
                        rendered.push('.');
                    }
                    if is_simple_field(field) {
                        rendered.push_str(field);
                    } else {
                        append_quoted_field(&mut rendered, field);
                    }
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
