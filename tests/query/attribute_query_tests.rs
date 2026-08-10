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
use qubit_model_metadata::FieldMetadata;
use qubit_model_metadata::TextConstraint;
use qubit_model_metadata::TextRepertoire;
use qubit_model_metadata::TypeRef;

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

#[test]
fn test_attribute_query_reads_field_attributes() {
    static ATTRIBUTES: [AttributeMetadata; 1] =
        [AttributeMetadata::Text(TextConstraint::new(
            None,
            Some(16),
            None,
            None,
            TextRepertoire::Ascii,
            false,
            None,
        ))];
    let field = FieldMetadata::new(
        0,
        "name",
        "String",
        TypeRef::of::<String>(),
        &ATTRIBUTES,
    );

    assert!(matches!(
        field.attribute(AttributeKind::Text),
        Some(AttributeMetadata::Text(_))
    ));
    assert_eq!(field.attributes_of(AttributeKind::Text).count(), 1);
    assert!(field.attribute(AttributeKind::Decimal).is_none());
}
