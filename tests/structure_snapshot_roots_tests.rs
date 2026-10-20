// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Root-scoped resolution against an explicitly frozen metadata snapshot.

#![cfg(feature = "generic")]

use qubit_model_derive::Model;
use qubit_model_metadata::__private::v8;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveErrorKind;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_reflect::ReflectionContext;

#[Model(id = "roots.SnapshotGood")]
struct SnapshotGood {
    value: String,
}

#[Model(id = "roots.SnapshotBad")]
struct SnapshotBad {
    #[validator(id = "roots.bad_snapshot", depends_on(expected(property = missing)))]
    value: String,
}

/// A frozen reflection snapshot isolates valid roots from unrelated defects.
#[test]
fn test_for_roots_ignores_unrelated_invalid_snapshot_member() {
    let reflection = ReflectionContext::builder()
        .include_type::<SnapshotGood>()
        .include_type::<SnapshotBad>()
        .include_type::<String>()
        .register(v8::model_metadata_registration::<SnapshotGood>())
        .register(v8::model_metadata_registration::<SnapshotBad>())
        .build()
        .expect("isolated reflection context");
    let models = ModelRegistry::from_context(&reflection).expect("snapshot model projection");
    let root = TypeMetadata::of::<SnapshotGood>();
    let roots = [root];
    let inputs = ResolveInputs {
        models: &models,
        roots: &roots,
    };

    let full_errors = StructureResolver::new(inputs)
        .resolve()
        .expect_err("complete audit visits the unrelated bad model");
    assert!(full_errors.errors().iter().any(|error| {
        error.kind() == ResolveErrorKind::MissingProperty
            && error.owner_type_id() == Some(TypeMetadata::of::<SnapshotBad>().type_id())
    }));

    let graph = StructureResolver::for_roots(inputs)
        .resolve()
        .expect("root-scoped resolution ignores the bad model");
    assert!(graph.model(root.type_id()).is_some());
    assert!(graph.model(TypeMetadata::of::<SnapshotBad>().type_id()).is_none());
}
