// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable local registries for multi-signature validator definitions.

// Contains the registry type and all of its implementations.
#[path = "validator_registry/validator_registry.rs"]
mod registry_impl;
pub use registry_impl::ValidatorRegistry;
