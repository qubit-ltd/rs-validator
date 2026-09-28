// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::ValidatorId;

#[test]
fn test_bind_error_preserves_safe_rule_parameter_and_dependency_context() {
    let error = BindError::new(BindErrorKind::MissingDependencyDeclaration)
        .with_rule(ValidatorId::new("test.rule"))
        .with_parameter("minimum")
        .with_dependency("other");

    assert_eq!(error.kind(), BindErrorKind::MissingDependencyDeclaration);
    assert_eq!(error.rule_id(), Some(ValidatorId::new("test.rule")));
    assert_eq!(error.parameter(), Some("minimum"));
    assert_eq!(error.dependency(), Some("other"));
    assert!(format!("{error:?}").contains("has_parameter"));
    assert!(error.to_string().contains("missing dependency declaration"));
}

#[test]
fn test_bind_error_kinds_have_stable_nonempty_display_values() {
    for kind in [
        BindErrorKind::UnknownParameter,
        BindErrorKind::DuplicateParameter,
        BindErrorKind::MissingParameter,
        BindErrorKind::ParameterTypeMismatch,
        BindErrorKind::ParameterAlreadyConsumed,
        BindErrorKind::ParameterOutOfRange,
        BindErrorKind::InvalidBounds,
        BindErrorKind::InvalidPattern,
        BindErrorKind::MissingRule,
        BindErrorKind::UnsupportedInput,
        BindErrorKind::MissingDependencyDeclaration,
        BindErrorKind::UnknownDependencyDeclaration,
        BindErrorKind::DependencyOrderMismatch,
        BindErrorKind::DependencyOptionalityMismatch,
        BindErrorKind::DependencyTypeMismatch,
        BindErrorKind::UnreadablePath,
        BindErrorKind::AmbiguousSignature,
        BindErrorKind::UnsupportedConstraint,
        BindErrorKind::MissingFeature,
        BindErrorKind::InvalidDeclaration,
        BindErrorKind::InvalidSelection,
        BindErrorKind::PreparedSignatureMismatch,
    ] {
        assert!(!kind.to_string().is_empty(), "{kind:?}");
    }
}
