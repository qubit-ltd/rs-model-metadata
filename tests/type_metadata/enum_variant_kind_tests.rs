// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for enum-variant structural metadata.

use qubit_model_metadata::AllowedChars;
use qubit_model_metadata::AttributeMetadata;
use qubit_model_metadata::EnumVariantKind;
use qubit_model_metadata::EnumVariantMetadata;
use qubit_model_metadata::FieldMetadata;
use qubit_model_metadata::TextConstraint;
use qubit_model_metadata::TypeRef;

static TEXT_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Text(TextConstraint::new(
    None,
    Some(32),
    None,
    None,
    AllowedChars::Unicode,
    false,
    None,
))];

static TUPLE_FIELDS: [FieldMetadata; 2] = [
    FieldMetadata::new(0, "0", "u8", TypeRef::of::<u8>(), &[]),
    FieldMetadata::new(
        1,
        "1",
        "Option<String>",
        TypeRef::of::<Option<String>>(),
        &TEXT_ATTRIBUTES,
    ),
];

static STRUCT_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(0, "message", "String", TypeRef::of::<String>(), &[])];

#[test]
fn test_enum_variant_kind_exposes_unit_shape() {
    let variant = EnumVariantMetadata::new(0, "UNIT");

    assert!(matches!(variant.kind(), EnumVariantKind::Unit));
    assert!(variant.fields().is_empty());
}

#[test]
fn test_enum_variant_kind_exposes_tuple_fields() {
    let variant = EnumVariantMetadata::tuple(1, "TUPLE", &TUPLE_FIELDS);

    assert!(matches!(variant.kind(), EnumVariantKind::Tuple(_)));
    assert_eq!(variant.fields().len(), 2);
    assert_eq!(variant.fields()[0].name(), "0");
    assert_eq!(variant.fields()[1].name(), "1");
    assert!(variant.fields()[1].is_nullable());
    assert_eq!(
        variant.fields()[1]
            .text_constraint()
            .and_then(|constraint| constraint.max_chars()),
        Some(32)
    );
}

#[test]
fn test_enum_variant_kind_exposes_struct_fields() {
    let variant = EnumVariantMetadata::structure(2, "STRUCT", &STRUCT_FIELDS);

    assert!(matches!(variant.kind(), EnumVariantKind::Struct(_)));
    assert_eq!(variant.fields().len(), 1);
    assert_eq!(variant.fields()[0].name(), "message");
}

#[test]
#[should_panic(expected = "field ordinals must match declaration order")]
fn test_enum_variant_kind_rejects_non_contiguous_field_ordinals() {
    static FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(1, "0", "u8", TypeRef::of::<u8>(), &[])];

    let _ = EnumVariantMetadata::tuple(0, "TUPLE", &FIELDS);
}

#[test]
#[should_panic(expected = "field names cannot be empty")]
fn test_enum_variant_kind_rejects_empty_field_names() {
    static FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(0, "", "u8", TypeRef::of::<u8>(), &[])];

    let _ = EnumVariantMetadata::structure(0, "STRUCT", &FIELDS);
}

#[test]
#[should_panic(expected = "variant fields cannot have duplicate names")]
fn test_enum_variant_kind_rejects_duplicate_field_names() {
    static FIELDS: [FieldMetadata; 2] = [
        FieldMetadata::new(0, "value", "u8", TypeRef::of::<u8>(), &[]),
        FieldMetadata::new(1, "value", "u16", TypeRef::of::<u16>(), &[]),
    ];

    let _ = EnumVariantMetadata::structure(0, "STRUCT", &FIELDS);
}
