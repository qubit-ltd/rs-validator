// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[test]
fn test_empty_registry_contains_no_registrations() {
    assert!(qubit_validator::ValidatorRegistry::empty().registrations().is_empty());
}
