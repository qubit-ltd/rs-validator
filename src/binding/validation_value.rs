//! Borrowed values passed across the erased validation boundary.

use std::any::Any;

/// A value view that deliberately avoids implicit stringification and cloning.
#[derive(Clone, Copy)]
pub enum ValidationValue<'a> {
    /// A borrowed UTF-8 string slice.
    Text(&'a str),
    /// A borrowed value with an exact concrete type available through `Any`.
    Typed(&'a dyn Any),
    /// An explicitly expanded optional value which is absent.
    Missing,
}

impl<'a> ValidationValue<'a> {
    /// Returns the concrete input shape, if one is present.
    #[must_use]
    pub fn input_type(self) -> Option<super::InputType> {
        match self {
            Self::Text(_) => Some(super::InputType::Text),
            Self::Typed(value) => {
                Some(super::InputType::Typed(value.type_id()))
            }
            Self::Missing => None,
        }
    }

    /// Returns the text view when this is a text value.
    #[must_use]
    pub const fn as_text(self) -> Option<&'a str> {
        match self {
            Self::Text(value) => Some(value),
            _ => None,
        }
    }

    /// Returns whether this value is the explicit missing marker.
    #[must_use]
    pub const fn is_missing(self) -> bool {
        matches!(self, Self::Missing)
    }

    /// Attempts to borrow the value as its exact concrete type.
    #[must_use]
    pub fn typed<T: 'static>(self) -> Option<&'a T> {
        match self {
            Self::Typed(value) => value.downcast_ref::<T>(),
            _ => None,
        }
    }
}

impl std::fmt::Debug for ValidationValue<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(_) => formatter.write_str("Text(<redacted>)"),
            Self::Typed(_) => formatter.write_str("Typed(<redacted>)"),
            Self::Missing => formatter.write_str("Missing"),
        }
    }
}
