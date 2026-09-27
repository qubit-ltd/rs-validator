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
///
/// # Examples
///
/// ```
/// use qubit_validator::{ValidatorId, Violation, ViolationCode};
///
/// let violation = Violation::new(
///     ValidatorId::new("text.required"),
///     ViolationCode::new("text.blank"),
/// );
/// assert_eq!(violation.code().as_str(), "text.blank");
/// ```
#[must_use]
#[derive(Clone, Eq, PartialEq)]
pub struct Violation {
    /// Stable identifier of the rule which rejected the value.
    rule_id: ValidatorId,
    /// Stable, program-declared violation code.
    code: ViolationCode,
    /// Structured location of the rejected value.
    path: ValidationPath,
    /// Structured parameters whose provenance must be checked by their caller.
    params: BTreeMap<&'static str, ViolationParam>,
}

impl Violation {
    /// Creates a safe violation at the root path.
    ///
    /// # Parameters
    ///
    /// - `rule_id`: Stable identifier of the rule that rejected the value.
    /// - `code`: Program-declared code describing the violation.
    ///
    /// # Returns
    ///
    /// A violation with an empty path and no parameters.
    #[inline]
    pub fn new(rule_id: ValidatorId, code: ViolationCode) -> Self {
        Self {
            rule_id,
            code,
            path: ValidationPath::root(),
            params: BTreeMap::new(),
        }
    }

    /// Replaces the structured location of the rejected value.
    ///
    /// # Parameters
    ///
    /// - `path`: Path containing only declared fields and opaque positions.
    ///
    /// # Returns
    ///
    /// The violation with `path` stored in place of its previous path.
    #[inline]
    pub fn with_path(mut self, path: ValidationPath) -> Self {
        self.path = path;
        self
    }

    /// Adds or replaces a structured parameter by its declared name.
    ///
    /// The type restricts representation but cannot prove provenance. Supply
    /// only program-declared tokens or rule configuration values, never
    /// rejected input or sensitive values derived from it.
    ///
    /// # Parameters
    ///
    /// - `name`: Static program-declared parameter name.
    /// - `value`: Scalar or static token associated with the name, subject to
    ///   the caller's provenance and redaction responsibility.
    ///
    /// # Returns
    ///
    /// The violation with the named parameter inserted or replaced.
    pub fn with_param(mut self, name: &'static str, value: ViolationParam) -> Self {
        self.params.insert(name, value);
        self
    }

    /// Returns the stable rule identifier.
    ///
    /// # Returns
    ///
    /// The identifier assigned by the bound validator.
    #[must_use]
    #[inline]
    pub const fn rule_id(&self) -> ValidatorId {
        self.rule_id
    }

    /// Returns the stable violation code.
    ///
    /// # Returns
    ///
    /// The program-declared code for this violation.
    #[must_use]
    #[inline]
    pub const fn code(&self) -> ViolationCode {
        self.code
    }

    /// Returns the structured path.
    ///
    /// The path contains no raw rejected value or map key.
    ///
    /// # Returns
    ///
    /// The structured path to the rejected value.
    #[must_use]
    #[inline]
    pub const fn path(&self) -> &ValidationPath {
        &self.path
    }

    /// Returns structured parameters supplied by the caller.
    ///
    /// The type restricts their representation but does not prove provenance;
    /// callers remain responsible for keeping rejected input and sensitive
    /// values derived from it out of this collection.
    ///
    /// # Returns
    ///
    /// The named parameters associated with this violation.
    #[must_use]
    #[inline]
    pub const fn params(&self) -> &BTreeMap<&'static str, ViolationParam> {
        &self.params
    }

    /// Assigns the bound rule identity to a safe adapter-produced draft.
    pub(crate) fn from_draft(rule_id: ValidatorId, draft: crate::ViolationDraft) -> Self {
        let (code, path, params) = draft.parts();
        Self {
            rule_id,
            code,
            path,
            params,
        }
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
