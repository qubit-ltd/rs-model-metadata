// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for attribute construction invariants.

mod attribute;

use qubit_model_metadata::AttributeKind;
use qubit_model_metadata::AttributeMetadata;
use qubit_model_metadata::DecimalConstraint;
use qubit_model_metadata::DecimalSemantic;
use qubit_model_metadata::ElementMetadata;
use qubit_model_metadata::FieldPath;
use qubit_model_metadata::IndexMetadata;
use qubit_model_metadata::KeyMetadata;
use qubit_model_metadata::LookupRelationMetadata;
use qubit_model_metadata::MapConstraint;
use qubit_model_metadata::ModelId;
use qubit_model_metadata::NamedTypeRef;
use qubit_model_metadata::OwnershipMetadata;
use qubit_model_metadata::PrimaryKeyFieldMetadata;
use qubit_model_metadata::PrimaryKeyMetadata;
use qubit_model_metadata::ReferenceMetadata;
use qubit_model_metadata::RoundingMode;
use qubit_model_metadata::SequenceConstraint;
use qubit_model_metadata::StrategyRef;
use qubit_model_metadata::TemporalConstraint;
use qubit_model_metadata::TemporalNormalization;
use qubit_model_metadata::TemporalPrecision;
use qubit_model_metadata::TextConstraint;
use qubit_model_metadata::TextRepertoire;
use qubit_model_metadata::TypeIdentity;
use qubit_model_metadata::UniqueComparison;
use qubit_model_metadata::UniqueFieldMetadata;
use qubit_model_metadata::UniqueMetadata;

const PRIMARY_KEY_FIELDS: [PrimaryKeyFieldMetadata; 1] = [PrimaryKeyFieldMetadata::new("id", true)];
const VALID_PRIMARY_KEY: PrimaryKeyMetadata = PrimaryKeyMetadata::new(&PRIMARY_KEY_FIELDS);
const UNIQUE_FIELDS: [UniqueFieldMetadata; 1] = [UniqueFieldMetadata::new("email", UniqueComparison::IgnoreCase)];
const VALID_UNIQUE: UniqueMetadata = UniqueMetadata::new(Some("user_email"), &UNIQUE_FIELDS);
const INDEX_FIELDS: [&str; 2] = ["organization_id", "email"];
const VALID_INDEX: IndexMetadata = IndexMetadata::new(Some("organization_email"), &INDEX_FIELDS);
const KEY_FIELDS: [&str; 1] = ["username"];
const VALID_KEY: KeyMetadata = KeyMetadata::new(Some("user"), &KEY_FIELDS);
const STRATEGY: StrategyRef = StrategyRef::new("redact-email");
const ELEMENT_TEXT: AttributeMetadata = AttributeMetadata::Text(TextConstraint::new(
    None,
    None,
    None,
    None,
    TextRepertoire::Ascii,
    false,
    None,
));
static DUPLICATE_ELEMENT_ATTRIBUTES: [AttributeMetadata; 2] = [ELEMENT_TEXT, ELEMENT_TEXT];
static INVALID_ELEMENT_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Codec(STRATEGY)];
const DUPLICATE_PRIMARY_KEY_FIELDS: [PrimaryKeyFieldMetadata; 2] = [
    PrimaryKeyFieldMetadata::new("id", false),
    PrimaryKeyFieldMetadata::new("id", true),
];
const DUPLICATE_UNIQUE_FIELDS: [UniqueFieldMetadata; 2] = [
    UniqueFieldMetadata::new("email", UniqueComparison::Exact),
    UniqueFieldMetadata::new("email", UniqueComparison::IgnoreCase),
];

#[test]
fn test_primary_key_constructor_remains_const_compatible() {
    assert_eq!(VALID_PRIMARY_KEY.fields().len(), 1);
}

#[test]
fn test_attribute_metadata_reports_its_kind() {
    let attribute = AttributeMetadata::Codec(STRATEGY);

    assert_eq!(attribute.kind(), AttributeKind::Codec);
}

#[test]
fn test_attribute_metadata_reports_every_kind() {
    let attributes = [
        AttributeMetadata::Text(TextConstraint::new(
            None,
            None,
            None,
            None,
            TextRepertoire::Unicode,
            false,
            None,
        )),
        AttributeMetadata::Sequence(SequenceConstraint::new(None, None, false)),
        AttributeMetadata::Map(MapConstraint::new(None, None)),
        AttributeMetadata::Temporal(TemporalConstraint::new(
            TemporalPrecision::Second,
            TemporalNormalization::Preserve,
        )),
        AttributeMetadata::Decimal(DecimalConstraint::new(
            None,
            0,
            RoundingMode::HalfUp,
            DecimalSemantic::Money,
        )),
        AttributeMetadata::Element(ElementMetadata::new(&[ELEMENT_TEXT])),
        AttributeMetadata::PrimaryKey(VALID_PRIMARY_KEY),
        AttributeMetadata::Unique(VALID_UNIQUE),
        AttributeMetadata::Index(VALID_INDEX),
        AttributeMetadata::Key(VALID_KEY),
        AttributeMetadata::Reference(ReferenceMetadata::new(
            ModelId::new("test.Target"),
            FieldPath::new(&["id"]),
            true,
            None,
        )),
        AttributeMetadata::LookupRelation(LookupRelationMetadata::new(
            NamedTypeRef::unresolved(TypeIdentity::of::<u64>()),
            FieldPath::new(&["id"]),
        )),
        AttributeMetadata::Ownership(OwnershipMetadata::new(NamedTypeRef::unresolved(
            TypeIdentity::of::<u64>(),
        ))),
        AttributeMetadata::Codec(STRATEGY),
        AttributeMetadata::Generator(StrategyRef::new("generate-id")),
    ];
    let expected = [
        AttributeKind::Text,
        AttributeKind::Sequence,
        AttributeKind::Map,
        AttributeKind::Temporal,
        AttributeKind::Decimal,
        AttributeKind::Element,
        AttributeKind::PrimaryKey,
        AttributeKind::Unique,
        AttributeKind::Index,
        AttributeKind::Key,
        AttributeKind::Reference,
        AttributeKind::LookupRelation,
        AttributeKind::Ownership,
        AttributeKind::Codec,
        AttributeKind::Generator,
    ];

    for (attribute, expected_kind) in attributes.into_iter().zip(expected) {
        assert_eq!(attribute.kind(), expected_kind);
    }
}

#[test]
fn test_metadata_accessors_return_declared_values() {
    let primary_key_field = VALID_PRIMARY_KEY
        .fields()
        .first()
        .expect("the valid primary key has one field");
    let unique_field = VALID_UNIQUE
        .fields()
        .first()
        .expect("the valid unique constraint has one field");

    assert!(VALID_PRIMARY_KEY.contains("id"));
    assert!(!VALID_PRIMARY_KEY.contains("missing"));
    assert_eq!(primary_key_field.name(), "id");
    assert!(primary_key_field.is_generated());

    assert_eq!(VALID_UNIQUE.name(), Some("user_email"));
    assert!(VALID_UNIQUE.contains("email"));
    assert_eq!(VALID_UNIQUE.comparison_of("email"), Some(UniqueComparison::IgnoreCase));
    assert_eq!(VALID_UNIQUE.comparison_of("missing"), None);
    assert_eq!(unique_field.name(), "email");
    assert_eq!(unique_field.comparison(), UniqueComparison::IgnoreCase);

    assert_eq!(VALID_INDEX.name(), Some("organization_email"));
    assert_eq!(VALID_INDEX.fields(), &INDEX_FIELDS);
    assert!(VALID_INDEX.contains("organization_id"));
    assert!(!VALID_INDEX.contains("missing"));

    assert_eq!(VALID_KEY.name(), Some("user"));
    assert_eq!(VALID_KEY.fields(), &KEY_FIELDS);
    assert!(VALID_KEY.contains("username"));
    assert!(!VALID_KEY.contains("missing"));

    assert_eq!(STRATEGY.name(), "redact-email");
}

#[test]
fn test_metadata_accessors_return_runtime_values() {
    let generated = PrimaryKeyFieldMetadata::new("created_id", false);
    let unique = UniqueFieldMetadata::new("display_name", UniqueComparison::Exact);

    assert_eq!(generated.name(), "created_id");
    assert!(!generated.is_generated());
    assert_eq!(unique.name(), "display_name");
    assert_eq!(unique.comparison(), UniqueComparison::Exact);
}

#[test]
#[should_panic(expected = "primary key requires at least one field")]
fn test_primary_key_rejects_empty_field_set() {
    let _ = PrimaryKeyMetadata::new(&[]);
}

#[test]
#[should_panic(expected = "unique constraint requires at least one field")]
fn test_unique_constraint_rejects_empty_field_set() {
    let _ = UniqueMetadata::new(None, &[]);
}

#[test]
#[should_panic(expected = "index requires at least one field")]
fn test_index_rejects_empty_field_set() {
    let _ = IndexMetadata::new(None, &[]);
}

#[test]
#[should_panic(expected = "logical key requires at least one field")]
fn test_logical_key_rejects_empty_field_set() {
    let _ = KeyMetadata::new(None, &[]);
}

#[test]
#[should_panic(expected = "constraint fields cannot contain duplicates")]
fn test_index_rejects_duplicate_field_names() {
    let _ = IndexMetadata::new(None, &["organization_id", "organization_id"]);
}

#[test]
#[should_panic(expected = "primary key fields cannot contain duplicates")]
fn test_primary_key_rejects_duplicate_field_names() {
    let _ = PrimaryKeyMetadata::new(&DUPLICATE_PRIMARY_KEY_FIELDS);
}

#[test]
#[should_panic(expected = "unique fields cannot contain duplicates")]
fn test_unique_constraint_rejects_duplicate_field_names() {
    let _ = UniqueMetadata::new(None, &DUPLICATE_UNIQUE_FIELDS);
}

#[test]
#[should_panic(expected = "logical constraint names cannot be empty")]
fn test_unique_constraint_rejects_empty_logical_name() {
    let _ = UniqueMetadata::new(Some(""), &UNIQUE_FIELDS);
}

#[test]
#[should_panic(expected = "logical constraint names cannot be empty")]
fn test_index_rejects_empty_logical_name() {
    let _ = IndexMetadata::new(Some(""), &INDEX_FIELDS);
}

#[test]
#[should_panic(expected = "logical constraint names cannot be empty")]
fn test_logical_key_rejects_empty_logical_name() {
    let _ = KeyMetadata::new(Some(""), &KEY_FIELDS);
}

#[test]
#[should_panic(expected = "strategy names cannot be empty")]
fn test_strategy_ref_rejects_empty_name() {
    let _ = StrategyRef::new("");
}

#[test]
#[should_panic(expected = "element metadata only supports text and decimal attributes")]
fn test_element_metadata_rejects_non_constraint_attributes() {
    let _ = ElementMetadata::new(&INVALID_ELEMENT_ATTRIBUTES);
}

#[test]
#[should_panic(expected = "element metadata attributes must be unique")]
fn test_element_metadata_rejects_duplicate_constraints() {
    let _ = ElementMetadata::new(&DUPLICATE_ELEMENT_ATTRIBUTES);
}
