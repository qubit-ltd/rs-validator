// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationOutcomeError;

#[test]
fn test_validation_outcome_rejects_empty_failure_data() {
    assert_eq!(
        ValidationOutcome::invalid(Vec::new()),
        Err(ValidationOutcomeError::EmptyViolations)
    );
    assert_eq!(
        ValidationOutcome::failed_prerequisite(Vec::new()),
        Err(ValidationOutcomeError::EmptyPrerequisites)
    );
}
