// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed values passed across the erased validation boundary.

use std::any::Any;

/// A value view that deliberately avoids implicit stringification and cloning.
///
/// # Examples
///
/// ```
/// use qubit_validator::ValidationValue;
///
/// let value = ValidationValue::Text("hello");
/// assert_eq!(value.as_text(), Some("hello"));
/// assert!(!value.is_missing());
/// ```
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum ValidationValue<'a> {
    /// A borrowed UTF-8 string slice.
    Text(
        /// Raw text borrowed only for the duration of validation.
        &'a str,
    ),
    /// A borrowed value with an exact concrete type available through `Any`.
    Typed(
        /// Raw typed value borrowed only for the duration of validation.
        &'a dyn Any,
    ),
    /// An explicitly expanded optional value which is absent.
    Missing,
}

impl<'a> ValidationValue<'a> {
    /// Returns the concrete input shape, if one is present.
    ///
    /// # Returns
    ///
    /// Returns `None` only for [`Self::Missing`].
    #[must_use]
    #[inline]
    pub fn input_type(self) -> Option<super::InputType> {
        match self {
            Self::Text(_) => Some(super::InputType::Text),
            Self::Typed(value) => Some(super::InputType::Typed(value.type_id())),
            Self::Missing => None,
        }
    }

    /// Returns the text view when this is a text value.
    ///
    /// # Returns
    ///
    /// Returns `None` for typed and missing values.
    #[must_use]
    #[inline]
    pub const fn as_text(self) -> Option<&'a str> {
        match self {
            Self::Text(value) => Some(value),
            _ => None,
        }
    }

    /// Returns whether this value is the explicit missing marker.
    ///
    /// # Returns
    ///
    /// Returns `true` only for [`Self::Missing`].
    #[must_use]
    #[inline]
    pub const fn is_missing(self) -> bool {
        matches!(self, Self::Missing)
    }

    /// Attempts to borrow the value as its exact concrete type.
    ///
    /// # Returns
    ///
    /// Returns `None` for text, missing, and differently typed values.
    #[must_use]
    #[inline]
    pub fn typed<T: 'static>(self) -> Option<&'a T> {
        match self {
            Self::Typed(value) => value.downcast_ref::<T>(),
            _ => None,
        }
    }
}

impl std::fmt::Debug for ValidationValue<'_> {
    /// Formats only the value shape and always redacts raw input.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(_) => formatter.write_str("Text(<redacted>)"),
            Self::Typed(_) => formatter.write_str("Typed(<redacted>)"),
            Self::Missing => formatter.write_str("Missing"),
        }
    }
}
