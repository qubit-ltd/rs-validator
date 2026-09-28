// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::SkipReason;
use qubit_validator::SkippedValidation;
use qubit_validator::ValidationPath;

#[test]
fn test_missing_optional_skip_preserves_occurrence_and_path() {
    let path = ValidationPath::root().with_field("optional");
    let skipped = SkippedValidation::missing_optional(3, path.clone());

    assert_eq!(skipped.occurrence(), 3);
    assert_eq!(skipped.reason(), SkipReason::MissingOptional);
    assert_eq!(skipped.path(), &path);
    assert!(skipped.prerequisites().is_empty());
}
