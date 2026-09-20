// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Execution error kinds.

/// A failure in the validation execution infrastructure.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExecutionErrorKind {
    /// The supplied input has the wrong erased type.
    InputTypeMismatch,
    /// A dependency has the wrong erased type.
    DependencyTypeMismatch,
    /// A required dependency value is absent.
    MissingRequiredDependencyValue,
    /// A property could not be read.
    PropertyReadFailed,
    /// Traversal exceeded an execution budget.
    TraversalLimit,
    /// A prepared adapter returned an invalid protocol result.
    AdapterContractViolation,
    /// An external operation failed.
    ExternalFailure,
}

impl std::fmt::Display for ExecutionErrorKind {
    /// Formats the stable human-readable error category.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::InputTypeMismatch => "input type mismatch",
            Self::DependencyTypeMismatch => "dependency type mismatch",
            Self::MissingRequiredDependencyValue => "missing required dependency value",
            Self::PropertyReadFailed => "property read failed",
            Self::TraversalLimit => "traversal limit exceeded",
            Self::AdapterContractViolation => "adapter contract violation",
            Self::ExternalFailure => "external failure",
        };
        formatter.write_str(name)
    }
}
