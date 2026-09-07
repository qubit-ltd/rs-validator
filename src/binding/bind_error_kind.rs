// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Binding error kinds.

/// A configuration or declaration error found while binding a rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindErrorKind {
    /// A parameter name is not known to the rule.
    UnknownParameter,
    /// A parameter was supplied more than once.
    DuplicateParameter,
    /// A required parameter is absent.
    MissingParameter,
    /// A parameter has the wrong value type.
    ParameterTypeMismatch,
    /// A parameter is outside the supported range.
    ParameterOutOfRange,
    /// Bounds are internally inconsistent.
    InvalidBounds,
    /// A pattern cannot be compiled.
    InvalidPattern,
    /// The requested rule is not registered.
    MissingRule,
    /// A registered rule cannot consume the requested input.
    UnsupportedInput,
    /// A dependency declaration is missing.
    MissingDependencyDeclaration,
    /// A dependency declaration is unknown.
    UnknownDependencyDeclaration,
    /// A dependency has the wrong type.
    DependencyTypeMismatch,
    /// A dependency path cannot be read.
    UnreadablePath,
    /// More than one signature matches the declaration.
    AmbiguousSignature,
    /// A declared constraint has no implementation.
    UnsupportedConstraint,
    /// An implementation feature is disabled.
    MissingFeature,
    /// A declaration is malformed.
    InvalidDeclaration,
    /// A requested selection is not known.
    InvalidSelection,
}

impl std::fmt::Display for BindErrorKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::UnknownParameter => "unknown parameter",
            Self::DuplicateParameter => "duplicate parameter",
            Self::MissingParameter => "missing parameter",
            Self::ParameterTypeMismatch => "parameter type mismatch",
            Self::ParameterOutOfRange => "parameter out of range",
            Self::InvalidBounds => "invalid bounds",
            Self::InvalidPattern => "invalid pattern",
            Self::MissingRule => "missing rule",
            Self::UnsupportedInput => "unsupported input",
            Self::MissingDependencyDeclaration => {
                "missing dependency declaration"
            }
            Self::UnknownDependencyDeclaration => {
                "unknown dependency declaration"
            }
            Self::DependencyTypeMismatch => "dependency type mismatch",
            Self::UnreadablePath => "unreadable path",
            Self::AmbiguousSignature => "ambiguous signature",
            Self::UnsupportedConstraint => "unsupported constraint",
            Self::MissingFeature => "missing feature",
            Self::InvalidDeclaration => "invalid declaration",
            Self::InvalidSelection => "invalid selection",
        };
        formatter.write_str(name)
    }
}
