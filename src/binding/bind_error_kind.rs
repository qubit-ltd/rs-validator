// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Binding error kinds.

/// A configuration or declaration error found while binding a rule.
///
/// # Examples
///
/// ```
/// use qubit_validator::BindErrorKind;
///
/// let kind = BindErrorKind::MissingParameter;
/// assert_eq!(kind.to_string(), "missing parameter");
/// ```
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BindErrorKind {
    /// A parameter name is not known to the rule.
    UnknownParameter,
    /// A parameter was supplied more than once.
    DuplicateParameter,
    /// A required parameter is absent.
    MissingParameter,
    /// A parameter has already been consumed by this reader.
    ParameterAlreadyConsumed,
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
    /// Dependency declarations do not follow the signature's slot order.
    DependencyOrderMismatch,
    /// A prepared implementation disagrees with its declared signature.
    PreparedSignatureMismatch,
    /// A required dependency may be absent on a statically known path.
    DependencyOptionalityMismatch,
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
    /// Formats the stable human-readable error category.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::UnknownParameter => "unknown parameter",
            Self::DuplicateParameter => "duplicate parameter",
            Self::MissingParameter => "missing parameter",
            Self::ParameterAlreadyConsumed => "parameter already consumed",
            Self::ParameterTypeMismatch => "parameter type mismatch",
            Self::ParameterOutOfRange => "parameter out of range",
            Self::InvalidBounds => "invalid bounds",
            Self::InvalidPattern => "invalid pattern",
            Self::MissingRule => "missing rule",
            Self::UnsupportedInput => "unsupported input",
            Self::MissingDependencyDeclaration => "missing dependency declaration",
            Self::UnknownDependencyDeclaration => "unknown dependency declaration",
            Self::DependencyOrderMismatch => "dependency order mismatch",
            Self::PreparedSignatureMismatch => "prepared signature mismatch",
            Self::DependencyOptionalityMismatch => "dependency optionality mismatch",
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
