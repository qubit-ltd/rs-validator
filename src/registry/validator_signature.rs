// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Input and dependency signatures for validator definitions.

use super::DependencySpec;
use super::InputType;
use super::PrepareFn;

/// One statically declared way to bind a validator.
#[derive(Clone, Copy)]
pub struct ValidatorSignature {
    /// Erased input shape accepted by this signature.
    input: InputType,
    /// Ordered dependency slots required by the prepared validator.
    dependencies: &'static [DependencySpec],
    /// Factory which decodes parameters and creates the prepared validator.
    prepare: PrepareFn,
}

impl ValidatorSignature {
    /// Creates a signature with ordered dependency slots.
    #[must_use]
    #[inline]
    pub const fn new(input: InputType, dependencies: &'static [DependencySpec], prepare: PrepareFn) -> Self {
        Self {
            input,
            dependencies,
            prepare,
        }
    }

    /// Returns the accepted value shape.
    #[must_use]
    #[inline]
    pub const fn input(self) -> InputType {
        self.input
    }

    /// Returns dependency declarations in slot order.
    #[must_use]
    #[inline]
    pub const fn dependencies(self) -> &'static [DependencySpec] {
        self.dependencies
    }

    /// Returns the parameter binding function.
    #[must_use]
    #[inline]
    pub const fn prepare(self) -> PrepareFn {
        self.prepare
    }
}

impl std::fmt::Debug for ValidatorSignature {
    /// Formats the input shape and dependency count without invoking the
    /// factory.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ValidatorSignature")
            .field("input", &self.input)
            .field("dependency_count", &self.dependencies.len())
            .finish()
    }
}
