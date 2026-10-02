// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;

use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::ValidationPath;
use qubit_validator::ValidatorId;

#[test]
fn test_execution_error_preserves_rule_and_safe_path_context() {
    let path = ValidationPath::root().with_field("secret field");
    let error = ExecutionError::new(ExecutionErrorKind::ExternalFailure)
        .with_rule(ValidatorId::new("test.rule"))
        .with_path(path.clone());

    assert_eq!(error.kind(), ExecutionErrorKind::ExternalFailure);
    assert_eq!(error.rule_id(), Some(ValidatorId::new("test.rule")));
    assert_eq!(error.path(), &path);
    assert!(Error::source(&error).is_none());
    assert!(!error.to_string().contains("secret field"));
    assert!(format!("{error:?}").contains("ExecutionError"));
}

#[test]
fn test_execution_error_kinds_have_stable_nonempty_display_values() {
    for kind in [
        ExecutionErrorKind::InputTypeMismatch,
        ExecutionErrorKind::DependencyTypeMismatch,
        ExecutionErrorKind::MissingRequiredDependencyValue,
        ExecutionErrorKind::DuplicateDependencyBinding,
        ExecutionErrorKind::UnknownDependencyBinding,
        ExecutionErrorKind::MissingDependencyBinding,
        ExecutionErrorKind::PropertyReadFailed,
        ExecutionErrorKind::TraversalLimit,
        ExecutionErrorKind::AdapterContractViolation,
        ExecutionErrorKind::ExternalFailure,
    ] {
        assert!(!kind.to_string().is_empty(), "{kind:?}");
    }
}

#[test]
fn test_invalid_selection_has_stable_display_and_preserves_path() {
    let path = ValidationPath::root().with_field("submitted_field");
    let error = ExecutionError::new(ExecutionErrorKind::InvalidSelection)
        .with_path(path.clone());

    assert_eq!(error.kind(), ExecutionErrorKind::InvalidSelection);
    assert_eq!(error.kind().to_string(), "invalid validation selection");
    assert_eq!(error.path(), &path);
}
