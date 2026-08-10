// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for static field paths and relation value objects.

mod relation;

use qubit_model_metadata::FieldPath;
use qubit_model_metadata::LookupRelationMetadata;
use qubit_model_metadata::ModelId;
use qubit_model_metadata::NamedTypeRef;
use qubit_model_metadata::OwnershipMetadata;
use qubit_model_metadata::ReferenceMetadata;
use qubit_model_metadata::TypeIdentity;

struct Target;

static EMPTY_PATH: FieldPath = FieldPath::new(&[]);
static EMPTY_SEGMENT_PATH: FieldPath = FieldPath::new(&[""]);

#[test]
fn test_field_path_preserves_static_segments() {
    let path = FieldPath::new(&["organization", "id"]);

    assert_eq!(path.segments(), &["organization", "id"]);
    assert!(!path.is_empty());
}

#[test]
fn test_field_path_reports_empty_segments() {
    assert!(FieldPath::new(&[]).is_empty());
}

#[test]
fn test_relation_metadata_preserves_targets_paths_and_policies() {
    let target = ModelId::from_static("test.metadata.Target");
    let named_target = NamedTypeRef::unresolved(TypeIdentity::of::<Target>());
    let target_field = FieldPath::new(&["id"]);
    let same_as = FieldPath::new(&["account_id"]);

    let reference =
        ReferenceMetadata::new(target, target_field, true, Some(same_as));
    assert_eq!(reference.target(), target);
    assert_eq!(reference.target_field(), target_field);
    assert!(reference.must_exist());
    assert_eq!(reference.same_as(), Some(same_as));

    let lookup = LookupRelationMetadata::new(named_target, target_field);
    assert_eq!(lookup.target().identity(), named_target.identity());
    assert_eq!(lookup.target_field(), target_field);

    let ownership = OwnershipMetadata::new(named_target);
    assert_eq!(ownership.owner().identity(), named_target.identity());
}

#[test]
#[should_panic(expected = "reference target field path cannot be empty")]
fn test_reference_rejects_empty_target_path() {
    let target = ModelId::from_static("test.metadata.Target");
    let _ = ReferenceMetadata::new(target, EMPTY_PATH, true, None);
}

#[test]
#[should_panic(
    expected = "reference target field path cannot contain empty segments"
)]
fn test_reference_rejects_empty_target_path_segments() {
    let target = ModelId::from_static("test.metadata.Target");
    let _ = ReferenceMetadata::new(target, EMPTY_SEGMENT_PATH, true, None);
}

#[test]
#[should_panic(expected = "lookup relation target field path cannot be empty")]
fn test_lookup_relation_rejects_empty_target_path() {
    let target = NamedTypeRef::unresolved(TypeIdentity::of::<Target>());
    let _ = LookupRelationMetadata::new(target, EMPTY_PATH);
}
