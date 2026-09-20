// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structured data violations.

use std::collections::BTreeMap;

use super::ValidationPath;
use super::ViolationCode;
use super::ViolationParam;
use crate::ValidatorId;

/// One rule violation without the rejected value.
#[must_use]
#[derive(Eq, PartialEq)]
pub struct Violation {
    /// Stable identifier of the rule which rejected the value.
    rule_id: ValidatorId,
    /// Stable, program-declared violation code.
    code: ViolationCode,
    /// Structured location of the rejected value.
    path: ValidationPath,
    /// Safe structured parameters with no raw rejected value.
    params: BTreeMap<&'static str, ViolationParam>,
}

impl Violation {
    /// Assigns a rule identifier to a safe adapter-produced draft.
    #[allow(dead_code)]
    pub(crate) fn from_draft(rule_id: ValidatorId, draft: crate::ViolationDraft) -> Self {
        let (code, path, params) = draft.parts();
        Self {
            rule_id,
            code,
            path,
            params,
        }
    }
    /// Creates a violation at the root path.
    #[inline]
    pub fn new(rule_id: ValidatorId, code: ViolationCode) -> Self {
        Self {
            rule_id,
            code,
            path: ValidationPath::root(),
            params: BTreeMap::new(),
        }
    }

    /// Replaces the violation path.
    #[inline]
    pub fn with_path(mut self, path: ValidationPath) -> Self {
        self.path = path;
        self
    }

    /// Adds or replaces a safe structured parameter.
    pub fn with_param(mut self, name: &'static str, value: ViolationParam) -> Self {
        self.params.insert(name, value);
        self
    }

    /// Returns the stable rule identifier.
    #[must_use]
    #[inline]
    pub const fn rule_id(&self) -> ValidatorId {
        self.rule_id
    }

    /// Returns the stable violation code.
    #[must_use]
    #[inline]
    pub const fn code(&self) -> ViolationCode {
        self.code
    }

    /// Returns the structured path.
    ///
    /// The path contains no raw rejected value or map key.
    #[must_use]
    #[inline]
    pub const fn path(&self) -> &ValidationPath {
        &self.path
    }

    /// Returns safe structured parameters.
    ///
    /// These parameters never contain the raw rejected input.
    #[must_use]
    #[inline]
    pub const fn params(&self) -> &BTreeMap<&'static str, ViolationParam> {
        &self.params
    }
}

impl std::fmt::Debug for Violation {
    /// Formats stable identifiers without exposing path labels or parameters.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Violation")
            .field("rule_id", &self.rule_id)
            .field("code", &self.code)
            .finish_non_exhaustive()
    }
}

impl std::fmt::Display for Violation {
    /// Formats only the stable violation code.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.code.fmt(formatter)
    }
}
