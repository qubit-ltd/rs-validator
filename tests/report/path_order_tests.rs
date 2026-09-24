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
use qubit_validator::ValidationPath;

fn hash(value: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn path_segment_order_is_consistent_with_equality() {
    let segments = [
        PathSegment::Field("a".into()),
        PathSegment::Field("b".into()),
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

#[test]
fn validation_path_order_is_consistent_with_equality() {
    let paths = [
        ValidationPath::root(),
        ValidationPath::root().with_field("a"),
        ValidationPath::root().with_field("b"),
        ValidationPath::root().with_index(0),
        ValidationPath::root().with_map_key(),
    ];

    for left in &paths {
        for right in &paths {
            assert_eq!(left == right, left.cmp(right).is_eq());
            if left == right {
                assert_eq!(hash(left), hash(right));
            }
        }
    }

    assert!(paths.windows(2).all(|pair| pair[0] < pair[1]));
}
