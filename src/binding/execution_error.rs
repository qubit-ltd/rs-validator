// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Structured validation execution failures.

use std::error::Error;

use super::ExecutionErrorKind;
use crate::ValidatorId;

/// An execution error with a safe public diagnostic surface.
pub struct ExecutionError {
    kind: ExecutionErrorKind,
    path: super::ValidationPath,
    rule_id: Option<ValidatorId>,
    source: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl ExecutionError {
    /// Creates an execution error at the root path.
    #[must_use]
    pub fn new(kind: ExecutionErrorKind) -> Self {
        Self {
            kind,
            path: super::ValidationPath::root(),
            rule_id: None,
            source: None,
        }
    }

    /// Associates a rule identifier with this error.
    #[must_use]
    pub const fn with_rule(mut self, rule_id: ValidatorId) -> Self {
        self.rule_id = Some(rule_id);
        self
    }

    /// Associates a structured path with this error.
    #[must_use]
    pub fn with_path(mut self, path: super::ValidationPath) -> Self {
        self.path = path;
        self
    }

    /// Retains an internal source error without exposing its text publicly.
    #[must_use]
    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }

    /// Returns the error kind.
    #[must_use]
    pub const fn kind(&self) -> ExecutionErrorKind {
        self.kind
    }

    /// Returns the associated rule identifier, if any.
    #[must_use]
    pub const fn rule_id(&self) -> Option<ValidatorId> {
        self.rule_id
    }

    /// Returns the structured error path.
    #[must_use]
    pub const fn path(&self) -> &super::ValidationPath {
        &self.path
    }

    /// Returns the retained internal source error.
    #[must_use]
    pub fn source(&self) -> Option<&(dyn Error + Send + Sync + 'static)> {
        self.source.as_deref()
    }
}

impl std::fmt::Debug for ExecutionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ExecutionError")
            .field("kind", &self.kind)
            .field("rule_id", &self.rule_id)
            .finish_non_exhaustive()
    }
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "validation execution failed: {}", self.kind)
    }
}

impl Error for ExecutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|source| source as &(dyn Error + 'static))
    }
}
