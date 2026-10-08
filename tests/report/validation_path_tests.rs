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

#[test]
fn test_validation_path_renders_special_fields_unambiguously() {
    let dotted_field = ValidationPath::root().with_field("a.b");
    let nested_fields = ValidationPath::root().with_field("a").with_field("b");
    let indexed_field = ValidationPath::root().with_field("a[2]");
    let field_then_index = ValidationPath::root().with_field("a").with_index(2);

    assert_eq!(dotted_field.render(), r#"["a.b"]"#);
    assert_eq!(nested_fields.render(), "a.b");
    assert_ne!(dotted_field.render(), nested_fields.render());

    assert_eq!(indexed_field.render(), r#"["a[2]"]"#);
    assert_eq!(field_then_index.render(), "a[2]");
    assert_ne!(indexed_field.render(), field_then_index.render());

    assert_eq!(ValidationPath::root().with_field("").render(), r#"[""]"#);
    assert_eq!(
        ValidationPath::root().with_field("<map-key>").render(),
        r#"["<map-key>"]"#
    );
    assert_ne!(
        ValidationPath::root().with_field("<map-key>").render(),
        ValidationPath::root().with_map_key().render()
    );
    assert_ne!(
        ValidationPath::root().with_field("<map-entry:0>").render(),
        ValidationPath::root().with_map_entry(0).render()
    );
    assert_ne!(
        ValidationPath::root().with_field("<map-value>").render(),
        ValidationPath::root().with_map_value().render()
    );
    assert_eq!(ValidationPath::root().with_field("x\\y").render(), r#"["x\\y"]"#);
    assert_eq!(ValidationPath::root().with_field("a\"b").render(), r#"["a\"b"]"#);
    assert_eq!(ValidationPath::root().with_field("a\n").render(), r#"["a\n"]"#);
    assert_eq!(
        ValidationPath::root().with_field("a\u{0001}").render(),
        r#"["a\u0001"]"#
    );
    assert_eq!(
        ValidationPath::root()
            .with_field("items")
            .with_index(2)
            .with_field("a.b")
            .render(),
        r#"items[2].["a.b"]"#
    );
}
