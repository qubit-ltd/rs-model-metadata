// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for structural type metadata validation.

use qubit_model_metadata::{
    AttributeMetadata,
    FieldMetadata,
    IndexMetadata,
    KeyMetadata,
    NamedTypeRef,
    OwnershipMetadata,
    StructMetadata,
    TypeIdentity,
    TypeKind,
    TypeMetadata,
    TypeRef,
};

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
