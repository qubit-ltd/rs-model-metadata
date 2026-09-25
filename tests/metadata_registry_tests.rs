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

use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_model_metadata::__private::ModelTypeSeal;
use qubit_model_metadata::__private::TypeMetadataProvider;
use qubit_model_metadata::__private::v7;
use qubit_model_metadata::__private::v7::register_model_capability;
use qubit_model_metadata::metadata::ModelId;
#[cfg(feature = "generic")]
use qubit_model_metadata::metadata::ModelRole;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::registry::ModelRegistryErrorKind;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectRegistry;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::identity::FragmentIdentity;

#[Model(id = "example.StaticPropertyFixture")]
struct StaticPropertyFixture {
    name: String,
}

#[ModelImpl]
impl StaticPropertyFixture {
    /// Returns the stored name through an independently registered getter.
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct RegistryFixture;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
#[cfg(feature = "generic")]
struct GenericFixture<T> {
    value: T,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
#[cfg(feature = "generic")]
struct OtherGenericFixture<T> {
    value: T,
}

#[Model]
#[cfg(feature = "generic")]
struct AnonymousGenericFixture<T> {
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
            let role = v7::leak(v7::model_role());
            v7::GeneratedTypeMetadataBuilder::new(
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
    let role = v7::leak(v7::model_role());
    let metadata = v7::leak(
        v7::GeneratedTypeMetadataBuilder::new(
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

fn local_provenance_metadata() -> &'static TypeMetadata {
    static METADATA: OnceLock<TypeMetadata> = OnceLock::new();
    METADATA.get_or_init(|| {
        let role = v7::leak(v7::model_role());
        v7::GeneratedTypeMetadataBuilder::new(
            TypeDescriptor::of::<RegistryFixture>(),
            Some(ModelId::new("example.LocalProvenance")),
            &[],
            role,
        )
        .finish::<RegistryFixture>()
    })
}

#[test]
fn explicit_registry_borrows_non_static_provenance() {
    let source = FragmentIdentity::new("fixture", "tests", line!(), 1, "model", 991);
    let registry = ModelRegistry::from_static_metadata(&[(local_provenance_metadata(), &source)]).expect("valid registry");
    let entry = registry.entries()[0];
    assert_eq!(entry.source(), &source);
    assert_eq!(entry.model_id().as_str(), "example.LocalProvenance");
    assert!(std::ptr::eq(
        entry.metadata().expect("concrete entry"),
        local_provenance_metadata()
    ));
}

#[test]
fn test_registry_indexes_registration_metadata_and_type_identity() {
    let item = entry("example.RegistryFixture", 1);
    let registry = ModelRegistry::from_static_metadata(&[item]).expect("valid registry");
    let entries = std::hint::black_box(ModelRegistry::entries);
    assert_eq!(entries(&registry).len(), 1);

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
    let error = ModelRegistry::from_static_metadata(&[second, first]).expect_err("duplicate IDs must fail");

    assert_eq!(error.kind(), ModelRegistryErrorKind::DuplicateModelId);
    assert_eq!(error.model_id().map(|id| id.as_str()), Some("example.Duplicate"));
    assert_eq!(error.sources().len(), 2);
    assert!(error.abi_cause().is_none());
    assert!(error.capability_id().is_none());
    assert!(error.expected_adapter_type().is_none());
    assert!(error.actual_adapter_type().is_none());
    assert_eq!(error.origins().len(), 2);
}

#[test]
#[cfg(feature = "generic")]
fn test_registry_indexes_one_generic_definition_without_concrete_model_id() {
    let concrete = TypeDescriptor::of::<GenericFixture<u8>>();
    let definition = concrete.type_definition().expect("generic definition");
    let generic = v7::leak(v7::generic_model_metadata(
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
    let registry = ModelRegistry::from_static_metadata_with_generics(&[], &[(generic, source)]).expect("generic registry");
    let generic_definitions = std::hint::black_box(ModelRegistry::generic_definitions);
    assert_eq!(generic_definitions(&registry).len(), 1);

    assert!(std::ptr::eq(
        registry.generic("example.GenericFixture").expect("generic lookup"),
        generic,
    ));
    assert!(registry.metadata("example.GenericFixture").is_none());
    assert_eq!(registry.generic_definitions().len(), 1);
    assert!(std::ptr::eq(
        registry
            .generic_metadata_for(definition.id())
            .expect("definition identity lookup"),
        generic,
    ));
    let isolated = ModelRegistry::from_static_metadata_with_generics(&[], &[]).expect("empty isolated registry");
    assert!(isolated.generic_metadata_for(definition.id()).is_none());
}

#[test]
#[cfg(feature = "generic")]
fn anonymous_generic_definition_is_queryable_but_not_an_id_entry() {
    let definition = TypeDescriptor::of::<GenericFixture<u8>>()
        .type_definition()
        .expect("generic definition");
    let generic = v7::leak(v7::generic_model_metadata(None, ModelRole::Model, definition, &[], &[]));
    let source = FragmentIdentity::new("fixture", "tests", 41, 1, "generic-model", 41);
    let registry = ModelRegistry::from_static_metadata_with_generics(&[], &[(generic, &source)]).expect("anonymous registry");

    assert!(registry.entries().is_empty());
    assert_eq!(registry.generic_definitions().len(), 1);
    assert!(std::ptr::eq(registry.generic_definitions()[0], generic));
    assert!(std::ptr::eq(
        registry
            .generic_metadata_for(definition.id())
            .expect("definition lookup"),
        generic,
    ));
    assert!(registry.generic("example.ArbitraryStableId").is_none());
}

#[test]
#[cfg(feature = "generic")]
fn anonymous_generic_definition_survives_reflection_projection() {
    let definition = TypeMetadata::of::<AnonymousGenericFixture<u8>>()
        .generic_definition()
        .expect("derived generic definition");
    let reflection = ReflectRegistry::initialize().expect("valid reflection registry");
    let registry = ModelRegistry::from_reflect_registry(reflection).expect("valid model projection");

    assert!(std::ptr::eq(
        registry
            .generic_metadata_for(definition.definition().id())
            .expect("definition lookup"),
        definition,
    ));
    assert!(
        registry
            .generic_definitions()
            .iter()
            .any(|candidate| std::ptr::eq(*candidate, definition))
    );
    assert!(registry.entries().iter().all(|entry| {
        entry
            .generic_metadata()
            .is_none_or(|candidate| !std::ptr::eq(candidate, definition))
    }));
}

#[test]
#[cfg(feature = "generic")]
fn generic_definitions_order_by_fragment_identity() {
    let first_definition = TypeDescriptor::of::<GenericFixture<u8>>()
        .type_definition()
        .expect("first generic definition");
    let second_definition = TypeDescriptor::of::<OtherGenericFixture<u8>>()
        .type_definition()
        .expect("second generic definition");
    let first = v7::leak(v7::generic_model_metadata(
        None,
        ModelRole::Model,
        first_definition,
        &[],
        &[],
    ));
    let second = v7::leak(v7::generic_model_metadata(
        None,
        ModelRole::Model,
        second_definition,
        &[],
        &[],
    ));
    let first_source = FragmentIdentity::new("fixture", "tests", 10, 1, "generic-model", 10);
    let second_source = FragmentIdentity::new("fixture", "tests", 20, 1, "generic-model", 20);
    let registry = ModelRegistry::from_static_metadata_with_generics(&[], &[(second, &second_source), (first, &first_source)])
        .expect("two anonymous generic definitions");

    assert_eq!(registry.generic_definitions().len(), 2);
    assert!(std::ptr::eq(registry.generic_definitions()[0], first));
    assert!(std::ptr::eq(registry.generic_definitions()[1], second));
}

#[test]
#[cfg(feature = "generic")]
fn duplicate_anonymous_generic_definition_reports_both_sources() {
    let definition = TypeDescriptor::of::<GenericFixture<u8>>()
        .type_definition()
        .expect("generic definition");
    let first = v7::leak(v7::generic_model_metadata(None, ModelRole::Model, definition, &[], &[]));
    let second = v7::leak(v7::generic_model_metadata(None, ModelRole::Model, definition, &[], &[]));
    let first_source = FragmentIdentity::new("fixture", "tests", 11, 1, "generic-model", 11);
    let second_source = FragmentIdentity::new("fixture", "tests", 12, 1, "generic-model", 12);
    let error = ModelRegistry::from_static_metadata_with_generics(&[], &[(first, &first_source), (second, &second_source)])
        .expect_err("duplicate definition identity must fail");

    assert_eq!(error.kind(), ModelRegistryErrorKind::RegistrationConflict);
    assert_eq!(error.sources().len(), 2);
    assert!(error.sources().contains(&first_source));
    assert!(error.sources().contains(&second_source));
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
    let registry = ModelRegistry::from_static_metadata(&[item]).expect("valid registry");
    let entries = registry.entries();
    assert_eq!(entries.len(), 1);
    let entry = registry.get("example.RegistryEntry").expect("entry");
    assert_eq!(entry.model_id().as_str(), "example.RegistryEntry");
    assert!(std::ptr::eq(entry.metadata().expect("concrete metadata"), item.0));
    assert!(std::ptr::eq(entry.source(), item.1));
    assert!(std::ptr::eq(
        registry.source("example.RegistryEntry").expect("registry source"),
        item.1,
    ));
    #[cfg(feature = "generic")]
    assert!(registry.generic("example.RegistryEntry").is_none());
    #[cfg(feature = "generic")]
    assert!(entry.generic_metadata().is_none());
}

/// Static constructors retain local properties while snapshots include impl getters.
#[test]
fn test_static_registry_excludes_independently_registered_model_impl_getter() {
    let metadata = TypeMetadata::of::<StaticPropertyFixture>();
    let source = FragmentIdentity::new("fixture", "tests", line!(), 1, "model", 992);
    let registry = ModelRegistry::from_static_metadata(&[(metadata, &source)]).expect("static registry");
    let properties = registry.properties_for(metadata).expect("static properties");
    assert!(properties.property("name").expect("stored property").getter().is_none());

    #[cfg(feature = "generic")]
    {
        let registry = ModelRegistry::from_static_metadata_with_generics(&[(metadata, &source)], &[])
            .expect("static registry with generic inputs");
        let properties = registry.properties_for(metadata).expect("static properties");
        assert!(properties.property("name").expect("stored property").getter().is_none());
    }

    let properties = ModelRegistry::try_global()
        .expect("global reflection registry")
        .properties_for(metadata)
        .expect("snapshot properties");
    let getter = properties
        .property("name")
        .expect("stored property")
        .getter()
        .expect("impl getter");
    assert_eq!(getter.rust_method_name(), "name");
}
