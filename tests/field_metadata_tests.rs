// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for field metadata validation and typed queries.

use qubit_model_metadata::AttributeMetadata;
use qubit_model_metadata::ElementMetadata;
use qubit_model_metadata::FieldMetadata;
use qubit_model_metadata::PrimaryKeyFieldMetadata;
use qubit_model_metadata::PrimaryKeyMetadata;
use qubit_model_metadata::SequenceConstraint;
use qubit_model_metadata::TextConstraint;
use qubit_model_metadata::TextRepertoire;
use qubit_model_metadata::TypeRef;

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
static ELEMENT_TEXT_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Text(TextConstraint::new(
        None,
        None,
        None,
        None,
        TextRepertoire::Ascii,
        false,
        None,
    ))];
static ELEMENT_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Element(ElementMetadata::new(
        &ELEMENT_TEXT_ATTRIBUTES,
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

#[test]
fn test_field_metadata_exposes_sequence_element_constraints() {
    let field = FieldMetadata::new(
        0,
        "aliases",
        "Vec<String>",
        TypeRef::of::<Vec<String>>(),
        &ELEMENT_ATTRIBUTES,
    );

    let element = field.element_metadata().expect("element metadata");
    assert!(matches!(
        element.attributes(),
        [AttributeMetadata::Text(constraint)]
            if constraint.repertoire() == TextRepertoire::Ascii
    ));
}

#[test]
#[should_panic(expected = "element attributes require a sequence field")]
fn test_field_metadata_rejects_element_constraints_on_scalar_fields() {
    let _ = FieldMetadata::new(
        0,
        "alias",
        "String",
        TypeRef::of::<String>(),
        &ELEMENT_ATTRIBUTES,
    );
}

#[test]
#[should_panic(expected = "text attributes require a text-capable element")]
fn test_field_metadata_rejects_constraints_unsupported_by_element_type() {
    let _ = FieldMetadata::new(
        0,
        "ids",
        "Vec<i64>",
        TypeRef::of::<Vec<i64>>(),
        &ELEMENT_ATTRIBUTES,
    );
}
