// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Rule-local violations before a bound validator assigns its identity.

use std::collections::BTreeMap;

use crate::ValidationPath;
use crate::ViolationCode;
use crate::ViolationParam;
/// A safe violation without a rule identity or raw rejected value.
#[must_use]
#[derive(Eq, PartialEq)]
pub struct ViolationDraft {
    /// Stable, program-declared violation code.
    code: ViolationCode,
    /// Structured location of the violation.
    path: ValidationPath,
    /// Safe structured parameters keyed by program-declared names.
    params: BTreeMap<&'static str, ViolationParam>,
}
impl ViolationDraft {
    /// Creates a draft at the root path.
    #[inline]
    pub fn new(code: ViolationCode) -> Self {
        Self {
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
    /// Adds a structured parameter.
    pub fn with_param(mut self, name: &'static str, value: ViolationParam) -> Self {
        self.params.insert(name, value);
        self
    }
    /// Splits the draft into safe components for final violation construction.
    pub(crate) fn parts(self) -> (ViolationCode, ValidationPath, BTreeMap<&'static str, ViolationParam>) {
        (self.code, self.path, self.params)
    }
}

impl std::fmt::Debug for ViolationDraft {
    /// Formats only safe metadata and the parameter count.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ViolationDraft")
            .field("code", &self.code)
            .field("path", &self.path)
            .field("parameter_count", &self.params.len())
            .finish()
    }
}
