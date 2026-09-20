// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Bounds applied while collecting validation results.

/// Explicit bounds applied while collecting a validation report.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ValidationLimits {
    /// Maximum violations; `None` means unlimited.
    pub max_violations: Option<usize>,
    /// Maximum skipped entries; `None` means unlimited.
    pub max_skipped: Option<usize>,
}
