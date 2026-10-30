// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable multi-signature validator descriptors.

// Defines the immutable descriptor type and its binding behavior.
#[path = "validator_descriptor/validator_descriptor.rs"]
mod descriptor_impl;

pub use descriptor_impl::ValidatorDescriptor;
