// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::ptr;

use qubit_validator::ArgumentReader;
use qubit_validator::BindErrorKind;
use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;

#[test]
fn test_required_argument_returns_raw_value_and_consumes_parameter() {
    let values = ["first", "second"];
    let args = [NamedValidationArgument::new(
        "choices",
        ValidationArgument::StringList(&values),
    )];
    let mut reader = ArgumentReader::new(&args).expect("argument names should be unique");

    assert_eq!(
        reader
            .required_argument("choices")
            .expect("present parameter should be returned"),
        ValidationArgument::StringList(&values)
    );
    reader.finish().expect("the parameter was consumed");

    let repeated = reader
        .required_argument("choices")
        .expect_err("second read should be rejected");
    assert_eq!(repeated.kind(), BindErrorKind::ParameterAlreadyConsumed);
    assert_eq!(repeated.parameter(), Some("choices"));

    let missing = reader
        .required_argument("missing")
        .expect_err("absent parameter should be rejected");
    assert_eq!(missing.kind(), BindErrorKind::MissingParameter);
    assert_eq!(missing.parameter(), Some("missing"));
}

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

#[test]
fn test_u32_readers_preserve_integer_boundaries_and_consumption() {
    for required in [true, false] {
        for value in [0_u128, u128::from(u32::MAX), u128::from(u32::MAX) + 1] {
            for argument in [
                ValidationArgument::Unsigned(value),
                ValidationArgument::Integer(i128::try_from(value).expect("u32 boundary fits i128")),
            ] {
                let args = [NamedValidationArgument::new("limit", argument)];
                let mut reader = ArgumentReader::new(&args).expect("unique parameter");
                let result = if required {
                    reader.required_u32("limit").map(Some)
                } else {
                    reader.optional_u32("limit")
                };
                let case = (required, value, matches!(argument, ValidationArgument::Integer(_)));
                match u32::try_from(value) {
                    Ok(expected) => assert_eq!(result.expect("valid u32 boundary"), Some(expected), "{case:?}"),
                    Err(_) => {
                        let error = result.expect_err("u32 overflow must be rejected");
                        assert_eq!(error.kind(), BindErrorKind::ParameterOutOfRange, "{case:?}");
                        assert_eq!(error.parameter(), Some("limit"), "{case:?}");
                    }
                }
                let repeated = reader.required_u32("limit").expect_err("present parameter is consumed");
                assert_eq!(repeated.kind(), BindErrorKind::ParameterAlreadyConsumed, "{case:?}");
                assert_eq!(repeated.parameter(), Some("limit"), "{case:?}");
                reader.finish().expect("even a failed conversion consumes its input");
            }
        }
    }
}

#[test]
fn test_optional_usize_decodes_values_and_reports_errors() {
    let cases: [(&str, ValidationArgument<'_>, Option<usize>, Option<BindErrorKind>); 5] = [
        (
            "unsigned usize max",
            ValidationArgument::Unsigned(usize::MAX as u128),
            Some(usize::MAX),
            None,
        ),
        (
            "integer usize max",
            ValidationArgument::Integer(usize::MAX as i128),
            Some(usize::MAX),
            None,
        ),
        (
            "unsigned overflow",
            ValidationArgument::Unsigned((usize::MAX as u128) + 1),
            None,
            Some(BindErrorKind::ParameterOutOfRange),
        ),
        (
            "negative integer",
            ValidationArgument::Integer(-1),
            None,
            Some(BindErrorKind::ParameterOutOfRange),
        ),
        (
            "boolean",
            ValidationArgument::Bool(true),
            None,
            Some(BindErrorKind::ParameterTypeMismatch),
        ),
    ];

    for (case, value, expected, expected_error) in cases {
        let args = [NamedValidationArgument::new("limit", value)];
        let mut reader = ArgumentReader::new(&args).expect("argument names should be unique");
        match expected_error {
            Some(expected_error) => {
                let error = reader.optional_usize("limit").expect_err("case expects an error");
                assert_eq!(error.kind(), expected_error, "{case}");
                assert_eq!(error.parameter(), Some("limit"), "{case}");
            }
            None => assert_eq!(
                reader.optional_usize("limit").expect("case expects a value"),
                expected,
                "{case}"
            ),
        }
    }
}

#[test]
fn test_optional_str_borrows_present_values_and_accepts_empty_strings() {
    let value = String::from("configured text");
    let empty = String::new();
    let args = [
        NamedValidationArgument::new("value", ValidationArgument::String(&value)),
        NamedValidationArgument::new("empty", ValidationArgument::String(&empty)),
    ];
    let mut reader = ArgumentReader::new(&args).expect("argument names should be unique");

    let decoded = reader
        .optional_str("value")
        .expect("string should decode")
        .expect("value is present");
    assert!(std::ptr::eq(decoded, value.as_str()));
    assert_eq!(
        reader.optional_str("empty").expect("empty string should decode"),
        Some("")
    );
    reader.finish().expect("both present parameters were consumed");
}

#[test]
fn test_optional_str_returns_none_for_absent_parameter() {
    let mut reader = ArgumentReader::new(&[]).expect("empty arguments should be valid");

    assert_eq!(
        reader.optional_str("label").expect("missing optional value is valid"),
        None
    );
    assert_eq!(reader.optional_str("label").expect("missing value stays absent"), None);
    reader.finish().expect("no parameter was left unread");
}

#[test]
fn test_optional_str_type_error_consumes_parameter() {
    let args = [NamedValidationArgument::new("label", ValidationArgument::Bool(true))];
    let mut reader = ArgumentReader::new(&args).expect("argument names should be unique");

    let error = reader.optional_str("label").expect_err("boolean is not a string");
    assert_eq!(error.kind(), BindErrorKind::ParameterTypeMismatch);
    assert_eq!(error.parameter(), Some("label"));

    let repeated = reader
        .optional_str("label")
        .expect_err("failed reads consume present parameters");
    assert_eq!(repeated.kind(), BindErrorKind::ParameterAlreadyConsumed);
    assert_eq!(repeated.parameter(), Some("label"));
    reader
        .finish()
        .expect("the failed typed read still consumed the parameter");
}

#[test]
fn test_optional_usize_handles_missing_and_repeated_reads() {
    let mut empty_reader = ArgumentReader::new(&[]).expect("empty arguments should be valid");
    assert_eq!(
        empty_reader
            .optional_usize("limit")
            .expect("missing value should be accepted"),
        None
    );

    let args = [NamedValidationArgument::new("limit", ValidationArgument::Unsigned(10))];
    let mut reader = ArgumentReader::new(&args).expect("argument names should be unique");
    assert_eq!(
        reader.optional_usize("limit").expect("first read should succeed"),
        Some(10)
    );
    let error = reader
        .optional_usize("limit")
        .expect_err("second read should be rejected");
    assert_eq!(error.kind(), BindErrorKind::ParameterAlreadyConsumed);
    assert_eq!(error.parameter(), Some("limit"));
}

#[test]
fn test_required_str_borrows_and_consumes_present_strings() {
    for value in [String::from("configured text"), String::new()] {
        let args = [NamedValidationArgument::new(
            "label",
            ValidationArgument::String(&value),
        )];
        let mut reader = ArgumentReader::new(&args).expect("unique parameter");
        let decoded = reader.required_str("label").expect("present string");
        assert_eq!(decoded, value.as_str());
        assert!(ptr::eq(decoded, value.as_str()), "the string must be borrowed");
        let error = reader.required_str("label").expect_err("present value was consumed");
        assert_eq!(error.kind(), BindErrorKind::ParameterAlreadyConsumed);
        assert_eq!(error.parameter(), Some("label"));
        reader.finish().expect("string was consumed");
    }
}

#[test]
fn test_required_str_reports_missing_and_consumes_type_errors() {
    let mut empty = ArgumentReader::new(&[]).expect("empty arguments");
    let missing = empty.required_str("label").expect_err("required string is absent");
    assert_eq!(missing.kind(), BindErrorKind::MissingParameter);
    assert_eq!(missing.parameter(), Some("label"));
    empty.finish().expect("no unread parameters");

    let args = [NamedValidationArgument::new("label", ValidationArgument::Bool(false))];
    let mut reader = ArgumentReader::new(&args).expect("unique parameter");
    let error = reader.required_str("label").expect_err("boolean is not a string");
    assert_eq!(error.kind(), BindErrorKind::ParameterTypeMismatch);
    assert_eq!(error.parameter(), Some("label"));
    let repeated = reader.required_str("label").expect_err("failed read consumed value");
    assert_eq!(repeated.kind(), BindErrorKind::ParameterAlreadyConsumed);
    assert_eq!(repeated.parameter(), Some("label"));
    reader.finish().expect("failed typed read consumed value");
}

#[test]
fn test_optional_bool_preserves_true_false_and_absence() {
    for value in [true, false] {
        let args = [NamedValidationArgument::new(
            "inclusive",
            ValidationArgument::Bool(value),
        )];
        let mut reader = ArgumentReader::new(&args).expect("unique parameter");
        assert_eq!(reader.optional_bool("inclusive").expect("boolean"), Some(value));
        let repeated = reader.optional_bool("inclusive").expect_err("present value consumed");
        assert_eq!(repeated.kind(), BindErrorKind::ParameterAlreadyConsumed);
        assert_eq!(repeated.parameter(), Some("inclusive"));
        reader.finish().expect("boolean consumed");
    }
    let mut empty = ArgumentReader::new(&[]).expect("empty arguments");
    assert_eq!(empty.optional_bool("inclusive").expect("absent value"), None);
    assert_eq!(empty.optional_bool("inclusive").expect("still absent"), None);
    empty.finish().expect("no unread parameters");
}

#[test]
fn test_optional_bool_type_error_consumes_present_parameter() {
    let args = [NamedValidationArgument::new(
        "inclusive",
        ValidationArgument::String("false"),
    )];
    let mut reader = ArgumentReader::new(&args).expect("unique parameter");
    let error = reader.optional_bool("inclusive").expect_err("string is not a boolean");
    assert_eq!(error.kind(), BindErrorKind::ParameterTypeMismatch);
    assert_eq!(error.parameter(), Some("inclusive"));
    let repeated = reader
        .optional_bool("inclusive")
        .expect_err("failed read consumed value");
    assert_eq!(repeated.kind(), BindErrorKind::ParameterAlreadyConsumed);
    assert_eq!(repeated.parameter(), Some("inclusive"));
    reader.finish().expect("failed typed read consumed value");
}
