// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;
#[test]
fn argument_debug_redacts_names_and_values() {
    let strings = ["first-secret", "second-secret"];
    let values = [
        ValidationArgument::String("sensitive-value"),
        ValidationArgument::StringList(&strings),
    ];
    let named = NamedValidationArgument::new("runtime-secret-name", values[0]);
    let output = format!("{values:?} {named:?}");
    for secret in [
        "first-secret",
        "second-secret",
        "sensitive-value",
        "runtime-secret-name",
    ] {
        assert!(!output.contains(secret), "debug output leaked {secret}");
    }
    assert!(output.contains("String(<redacted>)"));
    assert!(output.contains("StringList"));
    assert!(output.contains("len: 2"));
}
