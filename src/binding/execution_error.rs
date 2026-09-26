// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structured validation execution failures.

use super::ExecutionErrorKind;
use crate::ValidatorId;

/// An execution error with a safe public diagnostic surface.
///
/// Its public formatting and metadata never include raw validation input. An
/// underlying cause can be retained for an explicitly trusted diagnostic
/// consumer. The standard error chain deliberately does not expose that cause.
///
/// # Examples
///
/// ```
/// use qubit_validator::{ExecutionError, ExecutionErrorKind};
///
/// let error = ExecutionError::new(ExecutionErrorKind::InputTypeMismatch);
/// assert_eq!(error.kind(), ExecutionErrorKind::InputTypeMismatch);
/// assert!(std::error::Error::source(&error).is_none());
/// ```
#[must_use]
pub struct ExecutionError {
    /// Stable category describing the execution failure.
    kind: ExecutionErrorKind,
    /// Structured location of the failure, without rendered raw keys.
    path: super::ValidationPath,
    /// Optional rule identifier associated with the failure.
    rule_id: Option<ValidatorId>,
    /// Optional dependency name associated with the failure.
    dependency: Option<&'static str>,
    /// Optional cause available through the explicit trusted accessor.
    trusted_source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl ExecutionError {
    /// Creates an execution error at the root path.
    #[inline]
    pub fn new(kind: ExecutionErrorKind) -> Self {
        Self {
            kind,
            path: super::ValidationPath::root(),
            rule_id: None,
            dependency: None,
            trusted_source: None,
        }
    }

    /// Associates a rule identifier with this error.
    #[inline]
    pub const fn with_rule(mut self, rule_id: ValidatorId) -> Self {
        self.rule_id = Some(rule_id);
        self
    }

    /// Associates a dependency slot name with this error.
    #[inline]
    pub const fn with_dependency(mut self, dependency: &'static str) -> Self {
        self.dependency = Some(dependency);
        self
    }

    /// Retains an owned cause for an explicitly trusted diagnostic consumer.
    ///
    /// Ordinary formatting and [`std::error::Error::source`] never expose this
    /// value. Callers must avoid forwarding it to untrusted logs or clients.
    ///
    /// # Parameters
    ///
    /// - `source`: Owned, thread-safe source error retained for this failure.
    ///
    /// # Returns
    ///
    /// This error with the trusted cause attached.
    pub fn with_trusted_source<E>(mut self, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        self.trusted_source = Some(Box::new(source));
        self
    }

    /// Associates a structured path with this error.
    pub fn with_path(mut self, path: super::ValidationPath) -> Self {
        self.path = path;
        self
    }

    /// Returns the error kind.
    #[must_use = "the error category should be inspected"]
    #[inline]
    pub const fn kind(&self) -> ExecutionErrorKind {
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

    /// Returns the associated dependency slot name, if the error concerns one.
    ///
    /// # Returns
    ///
    /// Returns `Some` when a dependency has been attached, or `None` otherwise.
    #[must_use]
    #[inline]
    pub const fn dependency(&self) -> Option<&'static str> {
        self.dependency
    }

    /// Returns the structured error path.
    ///
    /// The path remains structured and does not expose the raw input value.
    #[must_use]
    #[inline]
    pub const fn path(&self) -> &super::ValidationPath {
        &self.path
    }

    /// Returns the retained cause to a caller that is trusted to inspect it.
    ///
    /// `None` means no owned cause was supplied. This accessor may reveal
    /// details unsuitable for ordinary logs or user-facing output.
    #[must_use]
    pub fn trusted_source(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)> {
        self.trusted_source.as_deref()
    }
}

impl std::fmt::Debug for ExecutionError {
    /// Formats structural metadata without exposing raw input or source text.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ExecutionError")
            .field("kind", &self.kind)
            .field("rule_id", &self.rule_id)
            .field("has_dependency", &self.dependency.is_some())
            .field("has_trusted_source", &self.trusted_source.is_some())
            .finish_non_exhaustive()
    }
}

impl std::fmt::Display for ExecutionError {
    /// Formats a stable summary without exposing raw input or source text.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "validation execution failed: {}", self.kind)
    }
}

impl std::error::Error for ExecutionError {}
