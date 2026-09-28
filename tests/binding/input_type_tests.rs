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
fn test_input_type_accepts_only_its_declared_value_shape() {
    let number = 4_u32;

    assert!(InputType::Text.accepts(ValidationValue::Text("value")));
    assert!(!InputType::Text.accepts(ValidationValue::Typed(&number)));
    assert!(!InputType::Text.accepts(ValidationValue::Missing));
    assert!(InputType::of::<u32>().accepts(ValidationValue::Typed(&number)));
    assert!(!InputType::of::<u64>().accepts(ValidationValue::Typed(&number)));
}
