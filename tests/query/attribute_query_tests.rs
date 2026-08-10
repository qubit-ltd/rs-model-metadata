// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for [`AttributeQuery`].

use qubit_model_metadata::AttributeKind;
use qubit_model_metadata::AttributeMetadata;
use qubit_model_metadata::AttributeQuery;

struct EmptyAttributes;

static EMPTY_ATTRIBUTES: [AttributeMetadata; 0] = [];

impl AttributeQuery for EmptyAttributes {
    fn attributes(&self) -> &'static [AttributeMetadata] {
        &EMPTY_ATTRIBUTES
    }
}

#[test]
fn test_attribute_query_returns_no_match_for_empty_attributes() {
    let query = EmptyAttributes;

    assert!(query.attribute(AttributeKind::PrimaryKey).is_none());
    assert_eq!(query.attributes_of(AttributeKind::PrimaryKey).count(), 0);
}
