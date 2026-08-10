// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::FieldMetadata;
use qubit_model_metadata::StructMetadata;
use qubit_model_metadata::TypeRef;

static DECLARED_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    0,
    "id",
    "i64",
    TypeRef::of::<i64>(),
    &[],
)];
static EMPTY_NAME_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    0,
    "",
    "i64",
    TypeRef::of::<i64>(),
    &[],
)];
static DUPLICATE_NAME_FIELDS: [FieldMetadata; 2] = [
    FieldMetadata::new(0, "id", "i64", TypeRef::of::<i64>(), &[]),
    FieldMetadata::new(1, "id", "i64", TypeRef::of::<i64>(), &[]),
];

#[test]
fn test_struct_metadata_exposes_empty_fields() {
    assert!(StructMetadata::new(&[]).fields().is_empty());
}

#[test]
fn test_struct_metadata_exposes_declared_fields() {
    let metadata = StructMetadata::new(&DECLARED_FIELDS);

    assert!(core::ptr::eq(metadata.fields().as_ptr(), DECLARED_FIELDS.as_ptr()));
    assert_eq!(metadata.fields().len(), DECLARED_FIELDS.len());
    assert_eq!(metadata.fields()[0].name(), "id");
}

#[test]
#[should_panic(expected = "field names cannot be empty")]
fn test_struct_metadata_rejects_empty_field_names() {
    let _ = StructMetadata::new(&EMPTY_NAME_FIELDS);
}

#[test]
#[should_panic(expected = "struct fields cannot have duplicate names")]
fn test_struct_metadata_rejects_duplicate_field_names() {
    let _ = StructMetadata::new(&DUPLICATE_NAME_FIELDS);
}
