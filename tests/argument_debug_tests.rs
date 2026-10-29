// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;

fn debug_shape(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_whitespace() && *character != ',')
        .collect()
}

#[test]
fn test_argument_debug_redacts_names_and_values() {
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

#[test]
fn test_all_argument_variants_redact_direct_and_named_debug() {
    let booleans = [true, false];
    let integers = [-731_i128, 947];
    let unsigned = [1731_u128, 2947];
    let strings = ["first-secret", "second-secret"];
    let cases = [
        (ValidationArgument::Bool(true), "Bool(<redacted>)"),
        (ValidationArgument::Bool(false), "Bool(<redacted>)"),
        (ValidationArgument::Integer(-731), "Integer(<redacted>)"),
        (ValidationArgument::Unsigned(1731), "Unsigned(<redacted>)"),
        (ValidationArgument::String("scalar-secret"), "String(<redacted>)"),
        (ValidationArgument::BoolList(&booleans), "BoolList { len: 2 }"),
        (ValidationArgument::IntegerList(&integers), "IntegerList { len: 2 }"),
        (ValidationArgument::UnsignedList(&unsigned), "UnsignedList { len: 2 }"),
        (ValidationArgument::StringList(&strings), "StringList { len: 2 }"),
    ];
    let name = String::from("runtime-secret-name");
    for (index, (argument, expected)) in cases.into_iter().enumerate() {
        let named = NamedValidationArgument::new(&name, argument);
        let expected_named = format!("NamedValidationArgument {{ value: {expected} }}");
        assert_eq!(format!("{argument:?}"), expected, "direct case {index}");
        assert_eq!(format!("{named:?}"), expected_named, "named case {index}");
        assert_eq!(
            debug_shape(&format!("{argument:#?}")),
            debug_shape(expected),
            "alternate direct case {index}"
        );
        assert_eq!(
            debug_shape(&format!("{named:#?}")),
            debug_shape(&expected_named),
            "alternate named case {index}"
        );
    }
}
