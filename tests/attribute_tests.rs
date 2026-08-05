// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for attribute construction invariants.

use qubit_model_metadata::{
    AttributeKind,
    AttributeMetadata,
    IndexMetadata,
    KeyMetadata,
    PrimaryKeyFieldMetadata,
    PrimaryKeyMetadata,
    SensitiveHandling,
    SensitiveMetadata,
    StrategyRef,
    UniqueComparison,
    UniqueFieldMetadata,
    UniqueMetadata,
};

const PRIMARY_KEY_FIELDS: [PrimaryKeyFieldMetadata; 1] =
    [PrimaryKeyFieldMetadata::new("id", true)];
const VALID_PRIMARY_KEY: PrimaryKeyMetadata =
    PrimaryKeyMetadata::new(&PRIMARY_KEY_FIELDS);
const UNIQUE_FIELDS: [UniqueFieldMetadata; 1] = [UniqueFieldMetadata::new(
    "email",
    UniqueComparison::IgnoreCase,
)];
const VALID_UNIQUE: UniqueMetadata =
    UniqueMetadata::new(Some("user_email"), &UNIQUE_FIELDS);
const INDEX_FIELDS: [&str; 2] = ["organization_id", "email"];
const VALID_INDEX: IndexMetadata =
    IndexMetadata::new(Some("organization_email"), &INDEX_FIELDS);
const KEY_FIELDS: [&str; 1] = ["username"];
const VALID_KEY: KeyMetadata = KeyMetadata::new(Some("user"), &KEY_FIELDS);
const STRATEGY: StrategyRef = StrategyRef::new("redact-email");
const SENSITIVE: SensitiveMetadata =
    SensitiveMetadata::new(SensitiveHandling::Mask);
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
    assert_eq!(
        VALID_UNIQUE.comparison_of("email"),
        Some(UniqueComparison::IgnoreCase)
    );
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
    assert_eq!(SENSITIVE.handling(), SensitiveHandling::Mask);
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
