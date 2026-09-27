// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Opaque identities for failures retained by one validation report.

use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

/// Identifies one retained failure within its originating report.
///
/// IDs are issued by [`super::ValidationReport::record_outcome`]. A caller
/// cannot construct an ID, and a report rejects IDs issued by another report.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FailureId {
    /// Process-local identity of the report that issued this failure ID.
    pub(crate) report: u64,
    /// Zero-based position of the retained violation in that report.
    pub(crate) index: usize,
}

impl FailureId {
    /// Creates an ID for a retained failure at `index` in `report`.
    ///
    /// The caller must ensure that the report identity and index refer to an
    /// entry already retained by the originating report.
    pub(crate) const fn new(report: u64, index: usize) -> Self {
        Self { report, index }
    }
}

/// Allocates a unique process-local identity for a new report.
///
/// # Panics
///
/// Panics only after the process has created `u64::MAX` validation reports.
pub(crate) fn next_report_id() -> u64 {
    static NEXT_REPORT_ID: AtomicU64 = AtomicU64::new(1);
    NEXT_REPORT_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| current.checked_add(1))
        .expect("validation report identity space exhausted")
}
