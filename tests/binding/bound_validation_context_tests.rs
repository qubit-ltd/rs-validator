// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::BoundValidationContext;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationValue;

#[test]
fn test_context_reads_values_and_paths_and_rejects_invalid_shapes() {
    let number = 9_u32;
    let values = [
        ValidationValue::Typed(&number),
        ValidationValue::Missing,
        ValidationValue::Text("text"),
    ];
    let context = BoundValidationContext::new(&values);

    assert_eq!(context.typed::<u32>(0).expect("typed slot matches"), &number);
    assert_eq!(
        context
            .optional_typed::<u32>(1)
            .expect("missing optional slot is valid"),
        None
    );
    assert_eq!(context.text(2).expect("text slot matches"), "text");
    assert_eq!(
        context.dependency_path(0).expect("slot exists"),
        &ValidationPath::root()
    );
    assert_eq!(context.value(0).expect("slot exists").typed::<u32>(), Some(&number));
    assert_eq!(
        context.typed::<u64>(0).expect_err("wrong type is rejected").kind(),
        ExecutionErrorKind::DependencyTypeMismatch
    );
    assert_eq!(
        context
            .typed::<u32>(1)
            .expect_err("required access rejects missing")
            .kind(),
        ExecutionErrorKind::MissingRequiredDependencyValue
    );
    assert_eq!(
        context.text(0).expect_err("text access rejects typed values").kind(),
        ExecutionErrorKind::DependencyTypeMismatch
    );
    assert_eq!(
        context.value(3).expect_err("unknown slots are rejected").kind(),
        ExecutionErrorKind::AdapterContractViolation
    );
    assert_eq!(
        context
            .dependency_path(3)
            .expect_err("unknown paths are rejected")
            .kind(),
        ExecutionErrorKind::AdapterContractViolation
    );

    let path = ValidationPath::root().with_field("dependency");
    let paths = [path.clone(), ValidationPath::root(), ValidationPath::root()];
    let with_paths = BoundValidationContext::new_with_paths(&values, &paths).expect("path count matches slots");
    assert_eq!(with_paths.dependency_path(0).expect("first path exists"), &path);
    assert_eq!(
        BoundValidationContext::new_with_paths(&values, &[])
            .expect_err("mismatched path count is invalid")
            .kind(),
        ExecutionErrorKind::AdapterContractViolation
    );
}
