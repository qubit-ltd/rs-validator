// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::PreparedOutcome;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;

#[test]
fn test_prepared_outcome_keeps_structured_violation_drafts() {
    let draft = ViolationDraft::new(ViolationCode::new("test.invalid"));

    assert!(matches!(
        PreparedOutcome::invalid(vec![draft]),
        Ok(PreparedOutcome::Invalid(_))
    ));
}
