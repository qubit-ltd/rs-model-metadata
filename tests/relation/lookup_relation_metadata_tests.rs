// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for [`LookupRelationMetadata`].

use qubit_model_metadata::FieldPath;
use qubit_model_metadata::LookupRelationMetadata;
use qubit_model_metadata::NamedTypeRef;
use qubit_model_metadata::TypeIdentity;

struct Target;

#[test]
fn test_lookup_relation_metadata_preserves_target_and_field() {
    let target = NamedTypeRef::unresolved(TypeIdentity::of::<Target>());
    let target_field = FieldPath::new(&["id"]);
    let lookup = LookupRelationMetadata::new(target, target_field);

    assert_eq!(lookup.target().identity(), target.identity());
    assert_eq!(lookup.target_field(), target_field);
}

#[test]
#[should_panic(expected = "lookup relation target field path cannot be empty")]
fn test_lookup_relation_rejects_empty_target_path() {
    let target = NamedTypeRef::unresolved(TypeIdentity::of::<Target>());
    let _ = LookupRelationMetadata::new(target, FieldPath::new(&[]));
}
