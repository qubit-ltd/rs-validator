// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::ValidationPath;
use qubit_validator::ValidatorId;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationParam;

#[test]
fn test_violation_exposes_safe_structured_fields() {
    let violation = Violation::new(ValidatorId::new("test.rule"), ViolationCode::new("test.invalid"))
        .with_path(ValidationPath::root().with_field("name"))
        .with_param("ok", ViolationParam::Bool(false))
        .with_param("count", ViolationParam::Unsigned(2));

    assert_eq!(violation.code().as_str(), "test.invalid");
    assert_eq!(violation.rule_id(), ValidatorId::new("test.rule"));
    assert_eq!(violation.path().render(), "name");
    assert_eq!(violation.params().len(), 2);
    assert_eq!(violation.to_string(), "test.invalid");
}
