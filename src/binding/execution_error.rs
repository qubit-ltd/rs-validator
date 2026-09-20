// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structured validation execution failures.

use std::error::Error;

use super::ExecutionErrorKind;
use crate::ValidatorId;

/// An execution error with a safe public diagnostic surface.
///
/// Its public formatting and metadata never include raw validation input.
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
    /// Internal source error retained for programmatic inspection.
    source: Option<Box<dyn Error + Send + Sync + 'static>>,
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
            source: None,
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

    /// Associates a structured path with this error.
    pub fn with_path(mut self, path: super::ValidationPath) -> Self {
        self.path = path;
        self
    }

    /// Retains an internal source error without exposing its text publicly.
    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
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

    /// Returns the retained internal source error.
    ///
    /// # Returns
    ///
    /// Returns `Some` when an internal source was retained, or `None`
    /// otherwise.
    #[must_use]
    #[inline]
    pub fn source(&self) -> Option<&(dyn Error + Send + Sync + 'static)> {
        self.source.as_deref()
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
            .finish_non_exhaustive()
    }
}

impl std::fmt::Display for ExecutionError {
    /// Formats a stable summary without exposing raw input or source text.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "validation execution failed: {}", self.kind)
    }
}

impl Error for ExecutionError {
    /// Returns the retained internal source through the standard error chain.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|source| source as &(dyn Error + 'static))
    }
}
