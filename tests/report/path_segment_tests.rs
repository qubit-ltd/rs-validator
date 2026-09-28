// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use qubit_validator::PathSegment;

fn hash(value: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn test_path_segment_order_and_hash_match_equality() {
    let segments = [
        PathSegment::Field("a"),
        PathSegment::Field("b"),
        PathSegment::Index(0),
        PathSegment::Index(1),
        PathSegment::MapEntry(0),
        PathSegment::MapKey,
        PathSegment::MapValue,
    ];

    for left in &segments {
        for right in &segments {
            assert_eq!(left == right, left.cmp(right).is_eq());
            if left == right {
                assert_eq!(hash(left), hash(right));
            }
        }
    }
    assert!(segments.windows(2).all(|pair| pair[0] < pair[1]));
}
