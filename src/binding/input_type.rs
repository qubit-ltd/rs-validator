// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Erased input shapes accepted by prepared validators.

use std::any::TypeId;

/// The two intentionally small input views used by the validation boundary.
///
/// # Examples
///
/// ```
/// use qubit_validator::{InputType, ValidationValue};
///
/// assert!(InputType::Text.accepts(ValidationValue::Text("hello")));
/// assert!(InputType::of::<u32>().accepts(ValidationValue::Typed(&3_u32)));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InputType {
    /// A borrowed UTF-8 text view.
    Text,
    /// A value whose concrete Rust type must match exactly.
    Typed(
        /// The exact concrete type accepted at the erased boundary.
        TypeId,
    ),
}

impl InputType {
    /// Creates a typed input descriptor for `T`.
    ///
    /// # Type Parameters
    ///
    /// - `T`: Exact concrete input type; it must be `'static` for `TypeId`.
    ///
    /// # Returns
    ///
    /// A descriptor that accepts only values whose concrete type is `T`.
    #[must_use]
    #[inline]
    pub const fn of<T: 'static>() -> Self {
        Self::Typed(TypeId::of::<T>())
    }

    /// Returns whether this descriptor accepts the supplied input shape.
    ///
    /// # Returns
    ///
    /// Returns `true` when `value` has the exact shape described by `self`.
    #[must_use]
    #[inline]
    pub fn accepts(self, value: super::ValidationValue<'_>) -> bool {
        match (self, value) {
            (Self::Text, super::ValidationValue::Text(_)) => true,
            (Self::Typed(expected), super::ValidationValue::Typed(value)) => value.type_id() == expected,
            _ => false,
        }
    }
}
