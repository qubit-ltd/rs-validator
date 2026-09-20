// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structured validation binding failures.

use super::BindErrorKind;
use crate::ValidatorId;

/// A binding error that does not retain raw parameter values.
#[must_use]
pub struct BindError {
    /// Stable category describing the failed binding operation.
    kind: BindErrorKind,
    /// Optional rule identifier associated with the failure.
    rule_id: Option<ValidatorId>,
    /// Optional parameter name; the parameter value is never retained.
    parameter: Option<String>,
    /// Optional dependency name; the dependency value is never retained.
    dependency: Option<String>,
}

impl BindError {
    /// Creates a binding error.
    #[inline]
    pub fn new(kind: BindErrorKind) -> Self {
        Self {
            kind,
            rule_id: None,
            parameter: None,
            dependency: None,
        }
    }

    /// Associates a rule identifier with this error.
    #[inline]
    pub const fn with_rule(mut self, rule_id: ValidatorId) -> Self {
        self.rule_id = Some(rule_id);
        self
    }

    /// Associates a parameter name with this error.
    pub fn with_parameter(mut self, parameter: impl Into<String>) -> Self {
        self.parameter = Some(parameter.into());
        self
    }

    /// Associates a dependency name with this error.
    pub fn with_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependency = Some(dependency.into());
        self
    }

    /// Returns the error kind.
    #[must_use = "the error category should be inspected"]
    #[inline]
    pub const fn kind(&self) -> BindErrorKind {
        self.kind
    }

    /// Returns the associated rule identifier, if any.
    ///
    /// # Returns
    ///
    /// Returns `Some` when a rule has been attached, or `None` otherwise.
    #[must_use]
    #[inline]
    pub const fn rule_id(&self) -> Option<ValidatorId> {
        self.rule_id
    }

    /// Returns the associated parameter name, if any.
    ///
    /// The error never retains the corresponding raw parameter value.
    ///
    /// # Returns
    ///
    /// Returns `Some` when a parameter has been attached, or `None` otherwise.
    #[must_use]
    #[inline]
    pub fn parameter(&self) -> Option<&str> {
        self.parameter.as_deref()
    }

    /// Returns the associated dependency name, if any.
    ///
    /// The error never retains the corresponding raw dependency value.
    ///
    /// # Returns
    ///
    /// Returns `Some` when a dependency has been attached, or `None` otherwise.
    #[must_use]
    #[inline]
    pub fn dependency(&self) -> Option<&str> {
        self.dependency.as_deref()
    }
}

impl std::fmt::Debug for BindError {
    /// Formats structural metadata without exposing raw parameter values.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BindError")
            .field("kind", &self.kind)
            .field("rule_id", &self.rule_id)
            .field("has_parameter", &self.parameter.is_some())
            .field("has_dependency", &self.dependency.is_some())
            .finish()
    }
}

impl std::fmt::Display for BindError {
    /// Formats a stable summary without exposing raw parameter values.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "validation binding failed: {}", self.kind)
    }
}

impl std::error::Error for BindError {}
