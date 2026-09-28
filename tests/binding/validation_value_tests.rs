// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::InputType;
use qubit_validator::ValidationValue;

#[test]
fn test_validation_value_reports_shape_without_exposing_contents() {
    let number = 4_u32;
    let text = ValidationValue::Text("private value");
    let typed = ValidationValue::Typed(&number);
    let missing = ValidationValue::Missing;

    assert_eq!(text.input_type(), Some(InputType::Text));
    assert_eq!(typed.input_type(), Some(InputType::of::<u32>()));
    assert_eq!(missing.input_type(), None);
    assert_eq!(text.as_text(), Some("private value"));
    assert_eq!(typed.typed::<u32>(), Some(&number));
    assert_eq!(typed.typed::<u64>(), None);
    assert!(missing.is_missing());
    assert!(format!("{text:?}").contains("redacted"));
    assert!(!format!("{text:?}").contains("private value"));
}
