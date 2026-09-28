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
fn test_validation_path_order_and_hash_match_equality() {
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

#[test]
fn test_validation_path_concatenation_preserves_segment_order() {
    let prefix = ValidationPath::root().with_field("person").with_index(2);
    let relative = ValidationPath::root().with_field("name");

    assert_eq!(
        prefix.concat(&relative).as_segments(),
        &[
            PathSegment::Field("person"),
            PathSegment::Index(2),
            PathSegment::Field("name"),
        ]
    );
}

#[test]
fn test_validation_path_renders_only_the_declared_safe_location() {
    let path = ValidationPath::root()
        .with_field("user")
        .with_index(2)
        .with_map_entry(3)
        .with_map_key()
        .with_map_value();

    assert_eq!(path.render(), "user[2].<map-entry:3>.<map-key>.<map-value>");
    assert_eq!(path.to_string(), "<validation-path>");
    assert!(format!("{path:?}").contains("segment_count"));
}
