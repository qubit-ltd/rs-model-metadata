// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for structural type metadata validation.

use qubit_model_metadata::AttributeMetadata;
use qubit_model_metadata::EnumMetadata;
use qubit_model_metadata::EnumVariantMetadata;
use qubit_model_metadata::FieldMetadata;
use qubit_model_metadata::IndexMetadata;
use qubit_model_metadata::KeyMetadata;
use qubit_model_metadata::NamedTypeRef;
use qubit_model_metadata::OwnershipMetadata;
use qubit_model_metadata::StructMetadata;
use qubit_model_metadata::TypeIdentity;
use qubit_model_metadata::TypeKind;
use qubit_model_metadata::TypeMetadata;
use qubit_model_metadata::TypeRef;

struct Account;
struct Organization;

static NON_CONTIGUOUS_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    1,
    "id",
    "i64",
    TypeRef::of::<i64>(),
    &[],
)];
static ACCOUNT_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    0,
    "id",
    "i64",
    TypeRef::of::<i64>(),
    &[],
)];
static INDEX_FIELDS: [&str; 1] = ["missing"];
static INVALID_INDEX_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Index(IndexMetadata::new(
        None,
        &INDEX_FIELDS,
    ))];
static KEY_FIELDS: [&str; 1] = ["id"];
static QUERY_ATTRIBUTES: [AttributeMetadata; 2] = [
    AttributeMetadata::Key(KeyMetadata::new(Some("account"), &KEY_FIELDS)),
    AttributeMetadata::Ownership(OwnershipMetadata::new(
        NamedTypeRef::unresolved(TypeIdentity::of::<Organization>()),
    )),
];

#[test]
#[should_panic(expected = "field ordinals must match declaration order")]
fn test_struct_metadata_rejects_non_contiguous_ordinals() {
    let _ = StructMetadata::new(&NON_CONTIGUOUS_FIELDS);
}

static ENUM_VARIANTS: [EnumVariantMetadata; 2] = [
    EnumVariantMetadata::new(0, "pending"),
    EnumVariantMetadata::new(1, "active"),
];
static EMPTY_NAME_VARIANTS: [EnumVariantMetadata; 1] =
    [EnumVariantMetadata::new(0, "")];
static DUPLICATE_NAME_VARIANTS: [EnumVariantMetadata; 2] = [
    EnumVariantMetadata::new(0, "pending"),
    EnumVariantMetadata::new(1, "pending"),
];
static NON_CONTIGUOUS_ENUM_VARIANTS: [EnumVariantMetadata; 1] =
    [EnumVariantMetadata::new(1, "pending")];

#[test]
#[should_panic(expected = "index references an unknown model field")]
fn test_type_metadata_rejects_constraints_for_missing_fields() {
    let _ = TypeMetadata::new(
        TypeIdentity::of::<Account>(),
        TypeKind::Struct(StructMetadata::new(&ACCOUNT_FIELDS)),
        &INVALID_INDEX_ATTRIBUTES,
    );
}

#[test]
fn test_type_metadata_exposes_keys_and_ownership() {
    let metadata = TypeMetadata::new(
        TypeIdentity::of::<Account>(),
        TypeKind::Struct(StructMetadata::new(&ACCOUNT_FIELDS)),
        &QUERY_ATTRIBUTES,
    );

    assert_eq!(
        metadata.keys().next().and_then(|key| key.name()),
        Some("account")
    );
    assert!(matches!(
        metadata.ownership(),
        Some(ownership)
            if ownership.owner().identity() == TypeIdentity::of::<Organization>()
    ));
}

#[test]
fn test_enum_metadata_queries_variants_by_name_and_ordinal() {
    let metadata = EnumMetadata::new(&ENUM_VARIANTS);

    assert_eq!(
        metadata.variant("pending").map(|variant| variant.ordinal()),
        Some(0)
    );
    assert!(metadata.variant("missing").is_none());
    assert_eq!(
        metadata.variant_at(1).map(|variant| variant.name()),
        Some("active")
    );
    assert!(metadata.variant_at(2).is_none());
}

#[test]
#[should_panic(expected = "enum variant names cannot be empty")]
fn test_enum_metadata_rejects_empty_variant_names() {
    let _ = EnumMetadata::new(&EMPTY_NAME_VARIANTS);
}

#[test]
#[should_panic(expected = "enum variant names must be unique")]
fn test_enum_metadata_rejects_duplicate_variant_names() {
    let _ = EnumMetadata::new(&DUPLICATE_NAME_VARIANTS);
}

#[test]
#[should_panic(expected = "enum variant ordinals must match declaration order")]
fn test_enum_metadata_rejects_non_contiguous_ordinals() {
    let _ = EnumMetadata::new(&NON_CONTIGUOUS_ENUM_VARIANTS);
}
