// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Crate-private behavior checks for locally assembled properties.

use qubit_reflect::__private::codegen_v4::expression::ExpressionData;
use qubit_reflect::__private::codegen_v4::expression::TypeExpressionData;

use crate::metadata::LocalPropertySet;
use crate::metadata::PropertyMetadata;

static VALUE_EXPRESSION: TypeExpressionData = ExpressionData::Named {
    path: "u32",
    definition: None,
    arguments: &[],
};

#[test]
fn test_empty_set_exposes_empty_slice_and_misses_every_name() {
    let set = LocalPropertySet::new(&[]);

    assert!(set.properties().is_empty());
    assert!(set.property("missing").is_none());
}

#[test]
fn test_property_lookup_matches_the_canonical_name() {
    let properties = Box::leak(
        vec![PropertyMetadata::new(
            "value",
            Some(&VALUE_EXPRESSION),
            None,
            None,
            None,
        )]
        .into_boxed_slice(),
    );
    let set = LocalPropertySet::new(properties);

    assert_eq!(set.property("value").map(PropertyMetadata::name), Some("value"));
    assert!(set.property("other").is_none());
}
