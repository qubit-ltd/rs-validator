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
///
/// # Examples
///
/// ```
/// use qubit_validator::{ViolationCode, ViolationDraft, ViolationParam};
///
/// let draft = ViolationDraft::new(ViolationCode::new("text.too_short"))
///     .with_param("minimum", ViolationParam::Unsigned(3));
/// assert!(format!("{draft:?}").contains("text.too_short"));
/// assert!(!format!("{draft:?}").contains("secret input"));
/// ```
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
    /// Creates a draft at the root path using a stable violation code.
    ///
    /// # Parameters
    /// - `code`: Program-declared code that identifies the validation failure.
    ///
    /// # Returns
    /// A draft with no parameters and an empty root path.
    #[inline]
    pub fn new(code: ViolationCode) -> Self {
        Self {
            code,
            path: ValidationPath::root(),
            params: BTreeMap::new(),
        }
    }
    /// Replaces the relative path attached to this violation draft.
    ///
    /// # Parameters
    /// - `path`: Structured location relative to the current validation target.
    ///
    /// # Returns
    /// The draft with `path` stored for later report prefixing.
    #[inline]
    pub fn with_path(mut self, path: ValidationPath) -> Self {
        self.path = path;
        self
    }
    /// Adds or replaces a safe structured parameter by its declared name.
    ///
    /// The value should describe a rule constraint or other program-defined
    /// metadata and must not contain rejected input.
    ///
    /// # Parameters
    /// - `name`: Static name declared by the application for this parameter.
    /// - `value`: Safe structured value to associate with `name`.
    ///
    /// # Returns
    /// The draft with the parameter inserted; an existing value with the same
    /// name is replaced.
    pub fn with_param(mut self, name: &'static str, value: ViolationParam) -> Self {
        self.params.insert(name, value);
        self
    }
    /// Splits the draft into safe components for final violation construction.
    ///
    /// The returned values are consumed by the binding layer, which supplies
    /// the stable rule identity before exposing a [`crate::Violation`].
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
