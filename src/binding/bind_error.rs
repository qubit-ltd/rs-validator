// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structured validation binding failures.

use std::error::Error;
use std::fmt;

use super::BindErrorKind;
use crate::ValidatorId;

/// A binding error that does not retain raw parameter values.
///
/// # Examples
///
/// ```
/// use qubit_validator::{BindError, BindErrorKind};
///
/// let error = BindError::new(BindErrorKind::MissingParameter).with_parameter("minimum");
/// assert_eq!(error.parameter(), Some("minimum"));
/// assert!(!error.to_string().contains("secret value"));
/// ```
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
    /// Creates a binding error for the supplied failure category.
    ///
    /// # Parameters
    ///
    /// - `kind`: Stable category describing why binding failed.
    ///
    /// # Returns
    ///
    /// A new error without an associated rule, parameter, or dependency.
    #[inline]
    pub fn new(kind: BindErrorKind) -> Self {
        Self {
            kind,
            rule_id: None,
            parameter: None,
            dependency: None,
        }
    }

    /// Returns the error kind.
    ///
    /// # Returns
    ///
    /// The stable category describing the binding failure.
    #[inline]
    pub const fn kind(&self) -> BindErrorKind {
        self.kind
    }

    /// Returns the resolved rule identifier attached to this error, if any.
    ///
    /// A registry lookup that fails with `MissingRule` has no resolved
    /// [`ValidatorId`], so this returns `None`. The caller retains the
    /// requested ID text when it needs to report which lookup failed.
    ///
    /// # Returns
    ///
    /// The resolved identifier for a selected rule, or `None` when no rule
    /// was selected.
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
    #[inline]
    pub fn dependency(&self) -> Option<&str> {
        self.dependency.as_deref()
    }

    /// Associates the resolved rule identifier with this error.
    ///
    /// # Parameters
    ///
    /// - `rule_id`: Identifier of the rule whose binding failed.
    ///
    /// # Returns
    ///
    /// This error with the identifier attached.
    #[inline]
    pub const fn with_rule(mut self, rule_id: ValidatorId) -> Self {
        self.rule_id = Some(rule_id);
        self
    }

    /// Associates a parameter name with this error without retaining its value.
    ///
    /// # Parameters
    ///
    /// - `parameter`: Name of the parameter involved in the failure.
    ///
    /// # Returns
    ///
    /// This error with the parameter name attached.
    pub fn with_parameter(mut self, parameter: impl Into<String>) -> Self {
        self.parameter = Some(parameter.into());
        self
    }

    /// Associates a dependency name with this error without retaining its
    /// value.
    ///
    /// # Parameters
    ///
    /// - `dependency`: Name of the dependency involved in the failure.
    ///
    /// # Returns
    ///
    /// This error with the dependency name attached.
    pub fn with_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependency = Some(dependency.into());
        self
    }
}

impl fmt::Debug for BindError {
    /// Formats structural metadata without exposing raw parameter values.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BindError")
            .field("kind", &self.kind)
            .field("rule_id", &self.rule_id)
            .field("has_parameter", &self.parameter.is_some())
            .field("has_dependency", &self.dependency.is_some())
            .finish()
    }
}

impl fmt::Display for BindError {
    /// Formats a stable summary without exposing raw parameter values.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "validation binding failed: {}", self.kind)
    }
}

impl Error for BindError {}
