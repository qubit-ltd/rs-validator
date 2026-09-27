// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Safe outcomes selected after domain validator failures.

use super::ViolationDraft;

/// The safe validation result selected after a domain validator returns an
/// error.
///
/// A mapper can deliberately ignore a domain error, or translate it into one
/// or more structured violations. It must not include raw rejected values in
/// violation drafts or other public diagnostics.
#[must_use]
#[non_exhaustive]
pub enum DomainErrorDisposition {
    /// Treat the domain error as non-fatal for validation.
    Valid,
    /// Treat the input as invalid with the supplied safe violation drafts.
    Invalid(Vec<ViolationDraft>),
}
