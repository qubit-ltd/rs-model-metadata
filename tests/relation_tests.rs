// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for static field paths and relation value objects.

use qubit_model_metadata::{
    FieldPath,
    LookupRelationMetadata,
    NamedTypeRef,
    OwnershipMetadata,
    ReferenceMetadata,
    TypeIdentity,
};

struct Target;

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
    let target = NamedTypeRef::unresolved(TypeIdentity::of::<Target>());
    let target_field = FieldPath::new(&["id"]);
    let same_as = FieldPath::new(&["account_id"]);

    let reference =
        ReferenceMetadata::new(target, target_field, true, Some(same_as));
    assert_eq!(reference.target().identity(), target.identity());
    assert_eq!(reference.target_field(), target_field);
    assert!(reference.must_exist());
    assert_eq!(reference.same_as(), Some(same_as));

    let lookup = LookupRelationMetadata::new(target, target_field);
    assert_eq!(lookup.target().identity(), target.identity());
    assert_eq!(lookup.target_field(), target_field);

    let ownership = OwnershipMetadata::new(target);
    assert_eq!(ownership.owner().identity(), target.identity());
}
