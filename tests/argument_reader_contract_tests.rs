// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::ArgumentReader;
use qubit_validator::BindErrorKind;
use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;

#[test]
fn test_required_parameter_cannot_be_read_twice() {
    let args = [NamedValidationArgument::new("limit", ValidationArgument::Unsigned(10))];
    let mut reader = ArgumentReader::new(&args).expect("argument names should be unique");

    assert_eq!(reader.required_u32("limit").expect("first read should succeed"), 10);
    let error = reader
        .required_u32("limit")
        .expect_err("second read should be rejected");

    assert_eq!(error.kind(), BindErrorKind::ParameterAlreadyConsumed);
    assert_eq!(error.parameter(), Some("limit"));
}

#[test]
fn test_present_optional_parameter_cannot_be_read_twice() {
    let args = [NamedValidationArgument::new("limit", ValidationArgument::Unsigned(10))];
    let mut reader = ArgumentReader::new(&args).expect("argument names should be unique");

    assert_eq!(
        reader.optional_u32("limit").expect("first read should succeed"),
        Some(10)
    );
    let error = reader
        .optional_u32("limit")
        .expect_err("second read should be rejected");

    assert_eq!(error.kind(), BindErrorKind::ParameterAlreadyConsumed);
    assert_eq!(error.parameter(), Some("limit"));
}

#[test]
fn test_missing_parameter_keeps_required_and_optional_semantics() {
    let mut reader = ArgumentReader::new(&[]).expect("empty arguments should be valid");

    assert_eq!(
        reader
            .required_u32("limit")
            .expect_err("missing required parameter should be rejected")
            .kind(),
        BindErrorKind::MissingParameter
    );
    assert_eq!(
        reader
            .optional_u32("limit")
            .expect("missing optional parameter should be accepted"),
        None
    );
}

#[test]
fn test_type_error_consumes_parameter() {
    let args = [NamedValidationArgument::new("limit", ValidationArgument::String("ten"))];
    let mut reader = ArgumentReader::new(&args).expect("argument names should be unique");

    assert_eq!(
        reader
            .required_u32("limit")
            .expect_err("wrong parameter type should be rejected")
            .kind(),
        BindErrorKind::ParameterTypeMismatch
    );
    let error = reader
        .required_str("limit")
        .expect_err("type failure should still consume the parameter");

    assert_eq!(error.kind(), BindErrorKind::ParameterAlreadyConsumed);
    assert_eq!(error.parameter(), Some("limit"));
}

#[test]
fn test_range_error_consumes_parameter() {
    let args = [NamedValidationArgument::new("limit", ValidationArgument::Integer(-1))];
    let mut reader = ArgumentReader::new(&args).expect("argument names should be unique");

    assert_eq!(
        reader
            .required_u32("limit")
            .expect_err("out-of-range parameter should be rejected")
            .kind(),
        BindErrorKind::ParameterOutOfRange
    );
    let error = reader
        .optional_u32("limit")
        .expect_err("range failure should still consume the parameter");

    assert_eq!(error.kind(), BindErrorKind::ParameterAlreadyConsumed);
    assert_eq!(error.parameter(), Some("limit"));
}
