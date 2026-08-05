// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for field metadata validation and typed queries.

use qubit_model_metadata::{
    AttributeMetadata,
    FieldMetadata,
    PrimaryKeyFieldMetadata,
    PrimaryKeyMetadata,
    SequenceConstraint,
    TextConstraint,
    TextRepertoire,
    TypeRef,
};

static INVALID_TEXT_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Text(TextConstraint::new(
        None,
        Some(32),
        None,
        None,
        TextRepertoire::Unicode,
        false,
        None,
    ))];
static PRIMARY_KEY_FIELDS: [PrimaryKeyFieldMetadata; 1] =
    [PrimaryKeyFieldMetadata::new("id", true)];
static INVALID_PRIMARY_KEY_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::PrimaryKey(PrimaryKeyMetadata::new(
        &PRIMARY_KEY_FIELDS,
    ))];
static SEQUENCE_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Sequence(SequenceConstraint::new(
        Some(1),
        Some(3),
        true,
    ))];

#[test]
fn test_field_metadata_exposes_declaration_details() {
    let field =
        FieldMetadata::new(2, "amount", "i64", TypeRef::of::<i64>(), &[]);

    assert_eq!(field.ordinal(), 2);
    assert_eq!(field.name(), "amount");
    assert_eq!(field.rust_type_name(), "i64");
    assert_eq!(
        field.field_type().type_name(),
        core::any::type_name::<i64>()
    );
    assert!(!field.is_nullable());
    assert!(field.attributes().is_empty());
}

#[test]
#[should_panic(expected = "text attributes require a text-capable field")]
fn test_field_metadata_rejects_text_constraints_on_non_text_types() {
    let _ = FieldMetadata::new(
        0,
        "id",
        "i64",
        TypeRef::of::<i64>(),
        &INVALID_TEXT_ATTRIBUTES,
    );
}

#[test]
#[should_panic(
    expected = "primary-key attributes are only valid at model scope"
)]
fn test_field_metadata_rejects_model_level_attributes() {
    let _ = FieldMetadata::new(
        0,
        "id",
        "i64",
        TypeRef::of::<i64>(),
        &INVALID_PRIMARY_KEY_ATTRIBUTES,
    );
}

#[test]
fn test_field_metadata_exposes_sequence_constraint() {
    let field = FieldMetadata::new(
        0,
        "tags",
        "Vec<String>",
        TypeRef::of::<Vec<String>>(),
        &SEQUENCE_ATTRIBUTES,
    );

    assert_eq!(
        field.sequence_constraint().map(|value| value.max_items()),
        Some(Some(3))
    );
}
