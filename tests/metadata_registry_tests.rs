// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow explicit-imports
//! Integration tests for frozen model registration indexes.

use std::sync::OnceLock;

use qubit_model_metadata::__private::ModelTypeSeal;
use qubit_model_metadata::__private::TypeMetadataProvider;
use qubit_model_metadata::__private::v4;
use qubit_model_metadata::__private::v4::register_model_capability;
use qubit_model_metadata::FragmentIdentity;
use qubit_model_metadata::ModelId;
use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::ModelRegistryErrorKind;
use qubit_model_metadata::ModelRole;
use qubit_model_metadata::Reflect;
use qubit_model_metadata::ReflectRegistry;
use qubit_model_metadata::TypeDescriptor;
use qubit_model_metadata::TypeMetadata;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct RegistryFixture;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct GenericFixture<T> {
    value: T,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct ProjectedFixture;

impl ModelTypeSeal for ProjectedFixture {}

impl TypeMetadataProvider for ProjectedFixture {
    fn __type_metadata() -> &'static TypeMetadata {
        static METADATA: OnceLock<TypeMetadata> = OnceLock::new();
        METADATA.get_or_init(|| {
            let role = v4::leak(v4::model_role());
            v4::GeneratedTypeMetadataBuilder::new(
                TypeDescriptor::of::<ProjectedFixture>(),
                Some(ModelId::new("example.ProjectedFixture")),
                &[],
                role,
            )
            .finish::<ProjectedFixture>()
        })
    }
}

register_model_capability!(ProjectedFixture, ProjectedFixture::__type_metadata);

fn entry(id: &'static str, fingerprint: u64) -> (&'static TypeMetadata, &'static FragmentIdentity) {
    let role = v4::leak(v4::model_role());
    let metadata = v4::leak(
        v4::GeneratedTypeMetadataBuilder::new(
            TypeDescriptor::of::<RegistryFixture>(),
            Some(ModelId::new(id)),
            &[],
            role,
        )
        .finish::<RegistryFixture>(),
    );
    let source = Box::leak(Box::new(FragmentIdentity::new(
        "fixture",
        "tests",
        fingerprint as u32,
        1,
        "model",
        fingerprint,
    )));
    (metadata, source)
}

#[test]
fn test_registry_indexes_registration_metadata_and_type_identity() {
    let item = entry("example.RegistryFixture", 1);
    let registry = ModelRegistry::from_metadata(&[item], &[]).expect("valid registry");

    assert!(std::ptr::eq(
        registry.metadata("example.RegistryFixture").expect("metadata"),
        item.0
    ));
    assert!(std::ptr::eq(
        registry
            .by_type_id(TypeDescriptor::of::<RegistryFixture>().type_id())
            .expect("type lookup"),
        item.0,
    ));
    assert!(registry.metadata("not-valid!").is_none());
}

#[test]
fn test_registry_reports_duplicate_ids_with_both_sources() {
    let first = entry("example.Duplicate", 1);
    let second = entry("example.Duplicate", 2);
    let error = ModelRegistry::from_metadata(&[second, first], &[]).expect_err("duplicate IDs must fail");

    assert_eq!(error.kind(), ModelRegistryErrorKind::DuplicateModelId);
    assert_eq!(error.model_id().map(|id| id.as_str()), Some("example.Duplicate"));
    assert_eq!(error.sources().len(), 2);
}

#[test]
fn test_registry_indexes_one_generic_definition_without_concrete_model_id() {
    let concrete = TypeDescriptor::of::<GenericFixture<u8>>();
    let definition = concrete.type_definition().expect("generic definition");
    let generic = v4::leak(v4::generic_model_metadata(
        ModelId::new("example.GenericFixture"),
        ModelRole::Model,
        definition,
        &[],
        &[],
    ));
    let source = Box::leak(Box::new(FragmentIdentity::new(
        "fixture",
        "tests",
        3,
        1,
        "generic-model",
        3,
    )));
    let registry = ModelRegistry::from_metadata(&[], &[(generic, source)]).expect("generic registry");

    assert!(std::ptr::eq(
        registry.generic("example.GenericFixture").expect("generic lookup"),
        generic,
    ));
    assert!(registry.metadata("example.GenericFixture").is_none());
    assert_eq!(registry.generic_definitions().len(), 1);
}

#[test]
fn test_registry_projects_concrete_models_and_sources_from_reflection() {
    let reflection = ReflectRegistry::initialize().expect("valid reflection registry");
    let registry = ModelRegistry::from_reflect_registry(reflection).expect("valid model projection");
    let reflected_source = reflection
        .type_source(TypeDescriptor::of::<ProjectedFixture>().type_id())
        .expect("reflected type source");

    assert!(std::ptr::eq(
        registry.source("example.ProjectedFixture").expect("projected source"),
        reflected_source,
    ));
    assert!(std::ptr::eq(
        registry
            .metadata("example.ProjectedFixture")
            .expect("concrete metadata"),
        ProjectedFixture::__type_metadata(),
    ));
}

/// Consumers can enumerate immutable model metadata and registration
/// provenance.
#[test]
fn test_registry_exposes_read_only_entries_with_sources() {
    let item = entry("example.RegistryEntry", 31);
    let registry = ModelRegistry::from_metadata(&[item], &[]).expect("valid registry");
    let entries = registry.entries();
    assert_eq!(entries.len(), 1);
    let entry = registry.get("example.RegistryEntry").expect("entry");
    assert_eq!(entry.model_id().as_str(), "example.RegistryEntry");
    assert!(std::ptr::eq(entry.metadata().expect("concrete metadata"), item.0));
    assert!(std::ptr::eq(entry.source(), item.1));
    assert!(entry.generic_metadata().is_none());
}
