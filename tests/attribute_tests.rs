// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for attribute construction invariants.

use qubit_model_metadata::{
    IndexMetadata,
    KeyMetadata,
    PrimaryKeyFieldMetadata,
    PrimaryKeyMetadata,
    UniqueMetadata,
};

const PRIMARY_KEY_FIELDS: [PrimaryKeyFieldMetadata; 1] =
    [PrimaryKeyFieldMetadata::new("id", true)];
const VALID_PRIMARY_KEY: PrimaryKeyMetadata =
    PrimaryKeyMetadata::new(&PRIMARY_KEY_FIELDS);

#[test]
fn test_primary_key_constructor_remains_const_compatible() {
    assert_eq!(VALID_PRIMARY_KEY.fields().len(), 1);
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
