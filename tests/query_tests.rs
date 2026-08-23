// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for static metadata queries.

mod query;

use core::any::TypeId;
use std::collections::HashMap;

use qubit_model_metadata::AttributeKind;
use qubit_model_metadata::AttributeMetadata;
use qubit_model_metadata::AttributeQuery;
use qubit_model_metadata::FieldMetadata;
use qubit_model_metadata::FieldPath;
use qubit_model_metadata::FieldPathResolveError;
use qubit_model_metadata::HasTypeMetadata;
use qubit_model_metadata::HasTypeShape;
use qubit_model_metadata::IndexMetadata;
use qubit_model_metadata::KeyMetadata;
use qubit_model_metadata::ModelId;
use qubit_model_metadata::NamedTypeRef;
use qubit_model_metadata::OwnershipMetadata;
use qubit_model_metadata::PrimaryKeyFieldMetadata;
use qubit_model_metadata::PrimaryKeyMetadata;
use qubit_model_metadata::StructMetadata;
use qubit_model_metadata::TextConstraint;
use qubit_model_metadata::TextRepertoire;
use qubit_model_metadata::TypeCapabilities;
use qubit_model_metadata::TypeIdentity;
use qubit_model_metadata::TypeKind;
use qubit_model_metadata::TypeMetadata;
use qubit_model_metadata::TypeRef;
use qubit_model_metadata::TypeShape;
use qubit_model_metadata::UniqueComparison;
use qubit_model_metadata::UniqueFieldMetadata;
use qubit_model_metadata::UniqueMetadata;
use qubit_model_metadata::metadata_of;

struct Account;
struct Contact;
struct Detached;
struct UnresolvedTarget;

static USERNAME_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Text(TextConstraint::new(
    None,
    Some(32),
    None,
    None,
    TextRepertoire::Unicode,
    false,
    None,
))];
static ACCOUNT_PRIMARY_KEY_FIELDS: [PrimaryKeyFieldMetadata; 1] = [PrimaryKeyFieldMetadata::new("id", true)];
static ACCOUNT_UNIQUE_FIELDS: [UniqueFieldMetadata; 1] =
    [UniqueFieldMetadata::new("username", UniqueComparison::IgnoreCase)];
static ACCOUNT_INDEX_FIELDS: [&str; 1] = ["username"];
static ACCOUNT_KEY_FIELDS: [&str; 1] = ["username"];
static ACCOUNT_ATTRIBUTES: [AttributeMetadata; 5] = [
    AttributeMetadata::PrimaryKey(PrimaryKeyMetadata::new(&ACCOUNT_PRIMARY_KEY_FIELDS)),
    AttributeMetadata::Unique(UniqueMetadata::new(None, &ACCOUNT_UNIQUE_FIELDS)),
    AttributeMetadata::Index(IndexMetadata::new(None, &ACCOUNT_INDEX_FIELDS)),
    AttributeMetadata::Key(KeyMetadata::new(None, &ACCOUNT_KEY_FIELDS)),
    AttributeMetadata::Ownership(OwnershipMetadata::new(NamedTypeRef::of::<Account>())),
];
static ACCOUNT_FIELDS: [FieldMetadata; 5] = [
    FieldMetadata::new(0, "id", "i64", TypeRef::of::<i64>(), &[]),
    FieldMetadata::new(1, "username", "String", TypeRef::of::<String>(), &USERNAME_ATTRIBUTES),
    FieldMetadata::new(
        2,
        "aliases",
        "Vec<Option<String>>",
        TypeRef::of::<Vec<Option<String>>>(),
        &[],
    ),
    FieldMetadata::new(
        3,
        "labels",
        "HashMap<String, String>",
        TypeRef::of::<HashMap<String, String>>(),
        &[],
    ),
    FieldMetadata::new(4, "contact", "Contact", TypeRef::of::<Contact>(), &[]),
];
static ACCOUNT_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.metadata.Account"),
    TypeIdentity::of::<Account>(),
    TypeKind::Struct(StructMetadata::new(&ACCOUNT_FIELDS)),
    &ACCOUNT_ATTRIBUTES,
);
static CONTACT_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(0, "email", "String", TypeRef::of::<String>(), &[])];
static CONTACT_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.metadata.Contact"),
    TypeIdentity::of::<Contact>(),
    TypeKind::Struct(StructMetadata::new(&CONTACT_FIELDS)),
    &[],
);
static DETACHED_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    0,
    "target",
    "UnresolvedTarget",
    TypeRef::of::<UnresolvedTarget>(),
    &[],
)];
static DETACHED_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.metadata.Detached"),
    TypeIdentity::of::<Detached>(),
    TypeKind::Struct(StructMetadata::new(&DETACHED_FIELDS)),
    &[],
);

impl HasTypeShape for Account {
    const TYPE_SHAPE: TypeShape = TypeShape::Named(NamedTypeRef::of::<Self>());
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::NONE;
}

impl HasTypeMetadata for Account {
    fn type_metadata() -> &'static TypeMetadata {
        &ACCOUNT_METADATA
    }
}

impl HasTypeShape for Contact {
    const TYPE_SHAPE: TypeShape = TypeShape::Named(NamedTypeRef::of::<Self>());
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::NONE;
}

impl HasTypeMetadata for Contact {
    fn type_metadata() -> &'static TypeMetadata {
        &CONTACT_METADATA
    }
}

impl HasTypeShape for Detached {
    const TYPE_SHAPE: TypeShape = TypeShape::Named(NamedTypeRef::of::<Self>());
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::NONE;
}

impl HasTypeMetadata for Detached {
    fn type_metadata() -> &'static TypeMetadata {
        &DETACHED_METADATA
    }
}

impl HasTypeShape for UnresolvedTarget {
    const TYPE_SHAPE: TypeShape = TypeShape::Named(NamedTypeRef::unresolved(TypeIdentity::of::<Self>()));
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::NONE;
}

#[test]
fn test_type_identity_exposes_rust_type_id() {
    let account = TypeIdentity::of::<Account>();

    assert_eq!(account.type_id(), TypeId::of::<Account>());
    assert_ne!(account.type_id(), TypeIdentity::of::<Contact>().type_id());
}

#[test]
fn test_field_and_unique_constraint_queries_are_typed() {
    let metadata = metadata_of::<Account>();
    let username = metadata.field("username").expect("username metadata");

    assert_eq!(
        metadata.fields().map(|field| field.name()).collect::<Vec<_>>(),
        vec!["id", "username", "aliases", "labels", "contact"]
    );
    assert!(!username.is_nullable());
    assert_eq!(username.text_constraint().and_then(|text| text.max_chars()), Some(32));
    assert_eq!(
        metadata
            .unique_constraints()
            .next()
            .and_then(|unique| unique.comparison_of("username")),
        Some(UniqueComparison::IgnoreCase)
    );
}

#[test]
fn test_primary_key_index_and_generic_attribute_queries_are_typed() {
    let metadata = metadata_of::<Account>();

    assert!(metadata.primary_key().is_some_and(|key| key.contains("id")));
    assert!(metadata.indexes().any(|index| index.contains("username")));
    assert_eq!(
        metadata.keys().next().map(|key| key.fields()),
        Some(&ACCOUNT_KEY_FIELDS[..])
    );
    assert_eq!(
        metadata.ownership().map(|ownership| ownership.owner().identity()),
        Some(TypeIdentity::of::<Account>())
    );
    assert!(matches!(
        metadata.attribute(AttributeKind::Unique),
        Some(AttributeMetadata::Unique(_))
    ));
    assert_eq!(metadata.attributes_of(AttributeKind::Index).count(), 1);
}

#[test]
fn test_vec_of_option_is_not_nullable_field() {
    let field = metadata_of::<Account>().field("aliases").expect("aliases metadata");

    assert!(!field.is_nullable());
    assert!(matches!(field.field_type().shape(), TypeShape::Sequence(element)
        if matches!(element.shape(), TypeShape::Optional(_))));
}

#[test]
fn test_map_field_preserves_key_and_value_shapes() {
    let field = metadata_of::<Account>().field("labels").expect("labels metadata");

    assert!(matches!(field.field_type().shape(), TypeShape::Map { key, value }
        if key.type_name() == core::any::type_name::<String>()
            && value.type_name() == core::any::type_name::<String>()));
}

#[test]
fn test_resolve_field_path_traverses_named_structs() {
    let path = FieldPath::new(&["contact", "email"]);

    assert_eq!(
        metadata_of::<Account>()
            .resolve_field_path(path)
            .expect("nested field")
            .name(),
        "email"
    );
}

#[test]
fn test_resolve_field_path_reports_missing_segment() {
    let path = FieldPath::new(&["contact", "unknown"]);

    assert!(matches!(
        metadata_of::<Account>().resolve_field_path(path),
        Err(FieldPathResolveError::FieldNotFound { segment: "unknown" })
    ));
}

#[test]
fn test_resolve_field_path_reports_non_struct_intermediate_segment() {
    let path = FieldPath::new(&["id", "value"]);

    assert!(matches!(
        metadata_of::<Account>().resolve_field_path(path),
        Err(FieldPathResolveError::IntermediateNotStruct { segment: "id" })
    ));
}

#[test]
fn test_resolve_field_path_reports_unresolvable_named_type() {
    let path = FieldPath::new(&["target", "value"]);

    assert!(matches!(
        metadata_of::<Detached>().resolve_field_path(path),
        Err(FieldPathResolveError::NamedMetadataUnavailable { segment: "target" })
    ));
}

#[test]
fn test_resolve_field_path_reports_empty_path() {
    assert!(matches!(
        metadata_of::<Account>().resolve_field_path(FieldPath::new(&[])),
        Err(FieldPathResolveError::EmptyPath)
    ));
}
