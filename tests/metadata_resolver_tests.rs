// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for explicit model graph resolution.

use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_derive::Model;
use qubit_model_derive::Value;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::registry::ModelRegistryOrigin;
use qubit_model_metadata::resolve::ResolveErrorKind;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_reflect::ReflectionContext;

#[Entity(id = "example.Target")]
struct TargetFixture {
    #[identifier]
    id: Id,
}

#[Model(id = "example.Source")]
struct SourceFixture {
    #[reference(entity = TargetFixture, property = id, existing = true)]
    target_id: Id,
}

#[Model(id = "example.SourceMissing")]
struct SourceMissingFixture {
    #[reference(entity = "missing.Target")]
    target: Id,
}

#[Value(id = "query.Nested")]
struct NestedQueryFixture {
    #[indexed]
    b: u64,
}

#[Entity(id = "query.Root")]
struct RootQueryFixture {
    #[identifier]
    id: Id,
    #[indexed]
    nested: NestedQueryFixture,
}

#[Entity(id = "query.Conflict")]
struct ConflictingQueryFixture {
    #[identifier]
    id: Id,
    #[indexed]
    a_b: u64,
    #[indexed]
    a: NestedQueryFixture,
}

#[Model(id = "closure.Model", eq, hash)]
struct PlainModelFixture {
    value: u64,
}

#[Value(id = "closure.Value")]
struct InvalidValueFixture {
    model: PlainModelFixture,
}

fn source_identity(module_path: &'static str, line: u32) -> &'static ModelRegistryOrigin {
    Box::leak(Box::new(ModelRegistryOrigin {
        crate_name: "fixture",
        crate_version: Some("1.0.0"),
        module_path,
        file: Some(file!()),
        line: Some(line),
        column: Some(1),
    }))
}

fn model_registry(
    entries: &[(&'static TypeMetadata, &'static ModelRegistryOrigin)],
) -> &'static ModelRegistry<'static> {
    Box::leak(Box::new(
        ModelRegistry::from_static_metadata(ReflectionContext::global(), entries)
            .expect("valid isolated model registry"),
    ))
}

#[test]
fn test_resolver_resolves_reference_targets_and_properties() {
    let target = TypeMetadata::of::<TargetFixture>();
    let source = TypeMetadata::of::<SourceFixture>();
    let registry = model_registry(&[
        (target, source_identity("fixture::target", 1)),
        (source, source_identity("fixture::source", 2)),
    ]);
    let graph = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: registry,
    })
    .resolve()
    .expect("complete source and target registry");

    let reference_field = source.field("target_id").unwrap();
    let resolved = graph
        .reference(reference_field.location().unwrap())
        .expect("resolved reference");
    assert!(std::ptr::eq(resolved.target(), target));
    assert_eq!(resolved.property().map(|property| property.name()), Some("id"));
    let query = graph.query(target.type_id()).expect("entity query");
    assert_eq!(query.declarations().len(), 1);
    assert_eq!(query.declarations()[0].path().segments(), &["id"]);
}

#[test]
fn test_resolver_aggregates_missing_targets_deterministically() {
    let source = TypeMetadata::of::<SourceMissingFixture>();
    let registry = model_registry(&[(source, source_identity("fixture::source_missing", 3))]);
    let errors = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: registry,
    })
    .resolve()
    .expect_err("missing targets must prevent graph publication");

    assert_eq!(errors.errors().len(), 1);
    assert_eq!(errors.errors()[0].kind(), ResolveErrorKind::MissingModelId);
    assert_eq!(errors.errors()[0].model_id(), Some("missing.Target"));
}

#[test]
fn test_query_recurses_indexed_value_fields_and_reports_flat_name_conflicts() {
    let nested = TypeMetadata::of::<NestedQueryFixture>();
    let root = TypeMetadata::of::<RootQueryFixture>();
    let registry = model_registry(&[
        (nested, source_identity("fixture::nested", 10)),
        (root, source_identity("fixture::root", 11)),
    ]);
    let graph = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: registry,
    })
    .resolve()
    .expect("indexed value field is a valid declaration");
    let query = graph.query(root.type_id()).unwrap();
    assert_eq!(
        query
            .declarations()
            .iter()
            .find(|declaration| declaration.field().name() == Some("nested"))
            .unwrap()
            .path()
            .segments(),
        &["nested"]
    );

    let conflict = TypeMetadata::of::<ConflictingQueryFixture>();
    let registry = model_registry(&[
        (nested, source_identity("fixture::nested", 10)),
        (conflict, source_identity("fixture::conflict", 12)),
    ]);
    let graph = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: registry,
    })
    .resolve()
    .expect("flat-name policy belongs to downstream consumers");
    assert_eq!(graph.query(conflict.type_id()).unwrap().declarations().len(), 3);
}

#[test]
fn test_resolver_rejects_value_closure_over_model_role() {
    let model = TypeMetadata::of::<PlainModelFixture>();
    let value = TypeMetadata::of::<InvalidValueFixture>();
    let registry = model_registry(&[
        (model, source_identity("fixture::model", 20)),
        (value, source_identity("fixture::value", 21)),
    ]);
    let errors = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: registry,
    })
    .resolve()
    .expect_err("a Value cannot contain a Model role");
    assert!(
        errors
            .errors()
            .iter()
            .any(|error| error.kind() == ResolveErrorKind::InvalidValueClosure)
    );
}
