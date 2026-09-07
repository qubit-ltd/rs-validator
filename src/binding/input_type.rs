//! Erased input shapes accepted by prepared validators.

use std::any::TypeId;

/// The two intentionally small input views used by the validation boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputType {
    /// A borrowed UTF-8 text view.
    Text,
    /// A value whose concrete Rust type must match exactly.
    Typed(TypeId),
}

impl InputType {
    /// Creates a typed input descriptor for `T`.
    #[must_use]
    pub const fn of<T: 'static>() -> Self {
        Self::Typed(TypeId::of::<T>())
    }

    /// Returns whether this descriptor accepts the supplied input shape.
    #[must_use]
    pub fn accepts(self, value: super::ValidationValue<'_>) -> bool {
        match (self, value) {
            (Self::Text, super::ValidationValue::Text(_)) => true,
            (Self::Typed(expected), super::ValidationValue::Typed(value)) => value.type_id() == expected,
            _ => false,
        }
    }
}
