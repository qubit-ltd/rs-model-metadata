// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for frozen model registration indexes.

use std::sync::OnceLock;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_model_metadata::__private::ModelMetadataProvider;
use qubit_model_metadata::__private::ModelTypeSeal;
use qubit_model_metadata::__private::TypeMetadataProvider;
use qubit_model_metadata::__private::v8;
use qubit_model_metadata::__private::v8::model_impl_registration;
use qubit_model_metadata::__private::v8::model_metadata_registration;
use qubit_model_metadata::metadata::ModelId;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::registry::ModelRegistryErrorKind;
use qubit_model_metadata::registry::ModelRegistryOrigin;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::capability::CapabilityRegistration;
use qubit_reflect::capability::CapabilitySpec;
use qubit_reflect::descriptor::Reflect as ReflectTrait;

#[Model(id = "example.StaticPropertyFixture")]
struct StaticPropertyFixture {
    name: String,
}

#[Model(id = "example.LinkedModelImplFixture")]
struct LinkedModelImplFixture {
    info: String,
}

#[ModelImpl]
impl LinkedModelImplFixture {
    pub fn info(&self) -> &str {
        &self.info
    }
}

#[ModelImpl]
impl StaticPropertyFixture {
    /// Returns the stored name through an independently registered getter.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Replaces the stored name through an independently registered setter.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

#[Reflect]
struct RegistryFixture;

#[Reflect]
struct StaticRegistryAbiFixture {
    value: String,
}

#[Model(id = "example.StaticRegistryAbiValid")]
struct StaticRegistryAbiValidFixture {
    value: String,
}

#[Reflect]
#[cfg(feature = "generic")]
struct StaticRegistryAbiWithGenericsFixture {
    value: String,
}

#[Model(id = "example.GenericFixture")]
#[cfg(feature = "generic")]
struct GenericFixture<T> {
    value: T,
}

#[Model(id = "example.OtherGenericFixture")]
#[cfg(feature = "generic")]
struct OtherGenericFixture<T> {
    value: T,
}

#[Model]
#[cfg(feature = "generic")]
struct AnonymousGenericFixture<T> {
    value: T,
}

#[Reflect]
struct ProjectedFixture;

impl ModelTypeSeal for ProjectedFixture {}

impl TypeMetadataProvider for ProjectedFixture {
    fn __type_metadata() -> &'static TypeMetadata {
        static METADATA: OnceLock<TypeMetadata> = OnceLock::new();
        METADATA.get_or_init(|| {
            let role = v8::leak(v8::model_role());
            v8::GeneratedTypeMetadataBuilder::new::<ProjectedFixture>(
                Some(ModelId::new("example.ProjectedFixture")),
                &[],
                role,
            )
            .finish::<ProjectedFixture>()
        })
    }
}

static COUNTED_PROVIDER_CALLS: AtomicUsize = AtomicUsize::new(0);

#[Reflect]
struct CountedProviderFixture;

impl ModelTypeSeal for CountedProviderFixture {}

impl TypeMetadataProvider for CountedProviderFixture {
    fn __type_metadata() -> &'static TypeMetadata {
        COUNTED_PROVIDER_CALLS.fetch_add(1, Ordering::SeqCst);
        static METADATA: OnceLock<TypeMetadata> = OnceLock::new();
        METADATA.get_or_init(|| {
            v8::GeneratedTypeMetadataBuilder::new::<CountedProviderFixture>(None, &[], v8::leak(v8::model_role()))
                .finish::<CountedProviderFixture>()
        })
    }
}

static NAMED_PROVIDER_CALLS: AtomicUsize = AtomicUsize::new(0);

#[Reflect]
struct CountedNamedProviderFixture;

impl ModelTypeSeal for CountedNamedProviderFixture {}

impl TypeMetadataProvider for CountedNamedProviderFixture {
    fn __type_metadata() -> &'static TypeMetadata {
        NAMED_PROVIDER_CALLS.fetch_add(1, Ordering::SeqCst);
        static METADATA: OnceLock<TypeMetadata> = OnceLock::new();
        METADATA.get_or_init(|| {
            v8::GeneratedTypeMetadataBuilder::new::<CountedNamedProviderFixture>(
                Some(ModelId::new("example.CountedNamedProvider")),
                &[],
                v8::leak(v8::model_role()),
            )
            .finish::<CountedNamedProviderFixture>()
        })
    }
}

#[Reflect]
struct SnapshotIsolatedFixture;

impl ModelTypeSeal for SnapshotIsolatedFixture {}

fn isolated_metadata(id: &'static str) -> &'static TypeMetadata {
    static FIRST: OnceLock<TypeMetadata> = OnceLock::new();
    static SECOND: OnceLock<TypeMetadata> = OnceLock::new();
    let cell = if id == "example.SnapshotFirst" { &FIRST } else { &SECOND };
    cell.get_or_init(|| {
        v8::GeneratedTypeMetadataBuilder::new::<SnapshotIsolatedFixture>(
            Some(ModelId::new(id)),
            &[],
            v8::leak(v8::model_role()),
        )
        .finish::<SnapshotIsolatedFixture>()
    })
}

fn first_snapshot_metadata() -> &'static TypeMetadata {
    isolated_metadata("example.SnapshotFirst")
}

fn second_snapshot_metadata() -> &'static TypeMetadata {
    isolated_metadata("example.SnapshotSecond")
}

struct TestModelMetadataCapability;

impl CapabilitySpec for TestModelMetadataCapability {
    type Adapter = ModelMetadataProvider;

    const ID: &'static str = "qubit.model.metadata.v1";
}

fn explicit_model_context<T: ReflectTrait + 'static>(provider: ModelMetadataProvider) -> ReflectionContext {
    ReflectionContext::builder()
        .include_type::<T>()
        .register(CapabilityRegistration::for_type::<T, TestModelMetadataCapability>(
            provider,
        ))
        .build()
        .expect("valid explicit model context")
}

fn entry(id: &'static str, fingerprint: u64) -> (&'static TypeMetadata, &'static ModelRegistryOrigin) {
    let role = v8::leak(v8::model_role());
    let metadata = v8::leak(
        v8::GeneratedTypeMetadataBuilder::new::<RegistryFixture>(Some(ModelId::new(id)), &[], role)
            .finish::<RegistryFixture>(),
    );
    let source = Box::leak(Box::new(registry_origin(
        "fixture",
        "tests",
        fingerprint as u32,
        1,
        "model",
        fingerprint,
    )));
    (metadata, source)
}

fn registry_origin(
    crate_name: &'static str,
    module_path: &'static str,
    line: u32,
    column: u32,
    _member_kind: &str,
    _fingerprint: u64,
) -> ModelRegistryOrigin {
    ModelRegistryOrigin {
        crate_name,
        crate_version: Some("0"),
        module_path,
        file: Some("tests/metadata_registry_tests.rs"),
        line: Some(line),
        column: Some(column),
    }
}

fn registry_descriptor<T: ReflectTrait + ?Sized + 'static>() -> TypeDescriptor<'static> {
    registry_context()
        .type_by_id(std::any::TypeId::of::<T>())
        .expect("registered test type")
}

fn registry_context() -> &'static ReflectionContext {
    static CONTEXT: OnceLock<ReflectionContext> = OnceLock::new();
    CONTEXT.get_or_init(|| {
        let builder = ReflectionContext::builder()
            .include_type::<RegistryFixture>()
            .include_type::<StaticRegistryAbiFixture>()
            .include_type::<StaticRegistryAbiValidFixture>()
            .include_type::<ProjectedFixture>()
            .include_type::<CountedProviderFixture>()
            .include_type::<CountedNamedProviderFixture>()
            .include_type::<SnapshotIsolatedFixture>()
            .include_type::<StaticPropertyFixture>()
            .include_type::<String>()
            .include_type::<str>();
        #[cfg(feature = "generic")]
        let builder = builder
            .include_type::<StaticRegistryAbiWithGenericsFixture>()
            .include_type::<GenericFixture<u8>>()
            .include_type::<OtherGenericFixture<u8>>()
            .include_type::<AnonymousGenericFixture<u8>>();
        builder.build().expect("valid registry test context")
    })
}

fn local_provenance_metadata() -> &'static TypeMetadata {
    static METADATA: OnceLock<TypeMetadata> = OnceLock::new();
    METADATA.get_or_init(|| {
        let role = v8::leak(v8::model_role());
        v8::GeneratedTypeMetadataBuilder::new::<RegistryFixture>(
            Some(ModelId::new("example.LocalProvenance")),
            &[],
            role,
        )
        .finish::<RegistryFixture>()
    })
}

#[test]
fn test_explicit_registry_retains_source_coordinates() {
    let source = registry_origin("fixture", "tests", line!(), 1, "model", 991);
    let registry = ModelRegistry::from_static_metadata(registry_context(), &[(local_provenance_metadata(), &source)])
        .expect("valid registry");
    let entry = &registry.entries()[0];
    assert_eq!(entry.source(), &source);
    assert_eq!(entry.declaration_source(), None);
    assert_eq!(entry.model_id().as_str(), "example.LocalProvenance");
    assert!(std::ptr::eq(
        entry.metadata().expect("concrete entry"),
        local_provenance_metadata()
    ));
}

#[test]
fn test_registry_indexes_registration_metadata_and_type_identity() {
    let item = entry("example.RegistryFixture", 1);
    let registry = ModelRegistry::from_static_metadata(registry_context(), &[item]).expect("valid registry");
    assert_eq!(registry.entries().len(), 1);

    assert!(std::ptr::eq(
        registry.metadata("example.RegistryFixture").expect("metadata"),
        item.0
    ));
    assert!(std::ptr::eq(
        registry
            .by_type_id(registry_descriptor::<RegistryFixture>().type_id())
            .expect("type lookup"),
        item.0,
    ));
    assert!(registry.metadata("not-valid!").is_none());
}

#[test]
fn test_registry_reports_duplicate_ids_with_both_sources() {
    let first = entry("example.Duplicate", 1);
    let second = entry("example.Duplicate", 2);
    let error =
        ModelRegistry::from_static_metadata(registry_context(), &[second, first]).expect_err("duplicate IDs must fail");

    assert_eq!(error.kind(), ModelRegistryErrorKind::DuplicateModelId);
    assert_eq!(error.model_id().map(|id| id.as_str()), Some("example.Duplicate"));
    assert_eq!(error.sources().len(), 2);
    assert!(error.abi_cause().is_none());
}

#[test]
fn test_static_registry_constructors_reject_invalid_concrete_abi() {
    let first_source = registry_origin("fixture", "tests", line!(), 1, "model", 992);
    #[cfg(feature = "generic")]
    let second_source = registry_origin("fixture", "tests", line!(), 1, "model", 993);
    let role = v8::leak(v8::model_role());
    let invalid = v8::leak(
        v8::GeneratedTypeMetadataBuilder::new::<StaticRegistryAbiFixture>(
            Some(ModelId::new("example.StaticRegistryAbi")),
            &[],
            role,
        )
        .finish_unchecked(),
    );
    #[cfg(feature = "generic")]
    let invalid_with_generics = v8::leak(
        v8::GeneratedTypeMetadataBuilder::new::<StaticRegistryAbiWithGenericsFixture>(
            Some(ModelId::new("example.StaticRegistryAbiWithGenerics")),
            &[],
            role,
        )
        .finish_unchecked(),
    );

    let error = ModelRegistry::from_static_metadata(registry_context(), &[(invalid, &first_source)])
        .expect_err("static registry must reject missing field overlays");
    assert_eq!(error.kind(), ModelRegistryErrorKind::InvalidMetadata);
    assert_eq!(error.abi_cause().expect("ABI cause").code(), "QMM-ABI-003");
    assert_eq!(error.sources(), std::slice::from_ref(&first_source));

    #[cfg(feature = "generic")]
    {
        let error = ModelRegistry::from_static_metadata_with_generics(
            registry_context(),
            &[(invalid_with_generics, &second_source)],
            &[],
        )
        .expect_err("static registry with generics must reject missing field overlays");
        assert_eq!(error.kind(), ModelRegistryErrorKind::InvalidMetadata);
        assert_eq!(error.abi_cause().expect("ABI cause").code(), "QMM-ABI-003");
        assert_eq!(error.sources(), std::slice::from_ref(&second_source));
    }

    let valid = TypeMetadata::of::<StaticRegistryAbiValidFixture>();
    assert!(ModelRegistry::from_static_metadata(registry_context(), &[(valid, &first_source)]).is_ok());
    #[cfg(feature = "generic")]
    assert!(
        ModelRegistry::from_static_metadata_with_generics(registry_context(), &[(valid, &second_source)], &[]).is_ok()
    );
}

#[test]
#[cfg(feature = "generic")]
fn test_registry_indexes_one_generic_definition_without_concrete_model_id() {
    let generic = TypeMetadata::of::<GenericFixture<u8>>()
        .generic_definition()
        .expect("generic metadata");
    let definition = generic.definition_in(registry_context()).expect("generic definition");
    let source = Box::leak(Box::new(registry_origin("fixture", "tests", 3, 1, "generic-model", 3)));
    let registry = ModelRegistry::from_static_metadata_with_generics(registry_context(), &[], &[(generic, source)])
        .expect("generic registry");
    assert_eq!(registry.generic_definitions().len(), 1);

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
    let isolated = ModelRegistry::from_static_metadata_with_generics(registry_context(), &[], &[])
        .expect("empty isolated registry");
    assert!(isolated.generic_metadata_for(definition.id()).is_none());
}

#[test]
#[cfg(feature = "generic")]
fn test_anonymous_generic_definition_is_queryable_but_not_an_id_entry() {
    let generic = TypeMetadata::of::<AnonymousGenericFixture<u8>>()
        .generic_definition()
        .expect("anonymous generic metadata");
    let definition = generic.definition_in(registry_context()).expect("generic definition");
    let source = registry_origin("fixture", "tests", 41, 1, "generic-model", 41);
    let registry = ModelRegistry::from_static_metadata_with_generics(registry_context(), &[], &[(generic, &source)])
        .expect("anonymous registry");

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
fn test_anonymous_generic_definition_survives_reflection_projection() {
    let definition = TypeMetadata::of::<AnonymousGenericFixture<u8>>()
        .generic_definition()
        .expect("derived generic definition");
    let reflection = ReflectionContext::global();
    let registry = ModelRegistry::from_context(reflection).expect("valid model projection");

    assert!(std::ptr::eq(
        registry
            .generic_metadata_for(
                definition
                    .definition_in(reflection)
                    .expect("context-bound definition")
                    .id()
            )
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
fn test_generic_definitions_order_by_registration_source() {
    let first = TypeMetadata::of::<GenericFixture<u8>>()
        .generic_definition()
        .expect("first generic metadata");
    let second = TypeMetadata::of::<OtherGenericFixture<u8>>()
        .generic_definition()
        .expect("second generic metadata");
    let first_source = registry_origin("fixture", "tests", 10, 1, "generic-model", 10);
    let second_source = registry_origin("fixture", "tests", 20, 1, "generic-model", 20);
    let registry = ModelRegistry::from_static_metadata_with_generics(
        registry_context(),
        &[],
        &[(second, &second_source), (first, &first_source)],
    )
    .expect("two anonymous generic definitions");

    assert_eq!(registry.generic_definitions().len(), 2);
    assert!(std::ptr::eq(registry.generic_definitions()[0], first));
    assert!(std::ptr::eq(registry.generic_definitions()[1], second));
}

#[test]
#[cfg(feature = "generic")]
fn test_duplicate_anonymous_generic_definition_reports_both_sources() {
    let generic = TypeMetadata::of::<AnonymousGenericFixture<u8>>()
        .generic_definition()
        .expect("anonymous generic metadata");
    let first_source = registry_origin("fixture", "tests", 11, 1, "generic-model", 11);
    let second_source = registry_origin("fixture", "tests", 12, 1, "generic-model", 12);
    let error = ModelRegistry::from_static_metadata_with_generics(
        registry_context(),
        &[],
        &[(generic, &first_source), (generic, &second_source)],
    )
    .expect_err("duplicate definition identity must fail");

    assert_eq!(error.kind(), ModelRegistryErrorKind::RegistrationConflict);
    assert_eq!(error.sources().len(), 2);
    assert!(error.sources().contains(&first_source));
    assert!(error.sources().contains(&second_source));
}

#[test]
fn test_registry_projects_concrete_models_and_sources_from_reflection() {
    let reflection = ReflectionContext::builder()
        .include_type::<ProjectedFixture>()
        .register(model_metadata_registration::<ProjectedFixture>())
        .build()
        .expect("valid explicit reflection context");
    let registry = ModelRegistry::from_context(&reflection).expect("valid model projection");
    let entry = registry.get("example.ProjectedFixture").expect("entry");
    assert_eq!(entry.source().crate_name, "explicit");
    assert!(entry.source().module_path.contains("ProjectedFixture"));
    assert_eq!(entry.declaration_source(), Some(entry.source()));
    assert!(std::ptr::eq(
        registry
            .metadata("example.ProjectedFixture")
            .expect("concrete metadata"),
        ProjectedFixture::__type_metadata(),
    ));
}

#[test]
fn test_combined_reflection_registration_projects_concrete_metadata() {
    let reflection = explicit_model_context::<ProjectedFixture>(ProjectedFixture::__type_metadata);
    let models = ModelRegistry::from_context(&reflection).expect("projected models");
    let by_id = models.metadata("example.ProjectedFixture").expect("metadata by ID");
    let by_type = models
        .metadata_for(
            &reflection
                .type_by_id(std::any::TypeId::of::<ProjectedFixture>())
                .expect("projected type"),
        )
        .expect("metadata by type")
        .expect("projected type");

    assert!(std::ptr::eq(by_id, by_type));
    assert!(std::ptr::eq(by_id, ProjectedFixture::__type_metadata()));
}

#[test]
fn test_metadata_for_caches_projected_and_anonymous_metadata_by_registry() {
    COUNTED_PROVIDER_CALLS.store(0, Ordering::SeqCst);
    let reflection = explicit_model_context::<CountedProviderFixture>(CountedProviderFixture::__type_metadata);
    let registry = ModelRegistry::from_context(&reflection).expect("projected registry");
    let descriptor = reflection
        .type_by_id(std::any::TypeId::of::<CountedProviderFixture>())
        .expect("type");
    let projected_calls = COUNTED_PROVIDER_CALLS.load(Ordering::SeqCst);
    let first = registry
        .metadata_for(&descriptor)
        .expect("first lookup")
        .expect("metadata");
    let after_first = COUNTED_PROVIDER_CALLS.load(Ordering::SeqCst);
    let second = registry
        .metadata_for(&descriptor)
        .expect("second lookup")
        .expect("metadata");
    let after_second = COUNTED_PROVIDER_CALLS.load(Ordering::SeqCst);

    assert!(std::ptr::eq(first, second));
    assert_eq!(after_first, projected_calls);
    assert_eq!(after_second, after_first);
    assert!(first.model_id().is_none(), "anonymous metadata is still cached");

    NAMED_PROVIDER_CALLS.store(0, Ordering::SeqCst);
    let named_reflection =
        explicit_model_context::<CountedNamedProviderFixture>(CountedNamedProviderFixture::__type_metadata);
    let named_registry = ModelRegistry::from_context(&named_reflection).expect("named registry");
    let named_descriptor = named_reflection
        .type_by_id(std::any::TypeId::of::<CountedNamedProviderFixture>())
        .expect("named type");
    let named_calls = NAMED_PROVIDER_CALLS.load(Ordering::SeqCst);
    let named_first = named_registry
        .metadata_for(&named_descriptor)
        .expect("named first lookup")
        .expect("named metadata");
    let named_second = named_registry
        .metadata_for(&named_descriptor)
        .expect("named second lookup")
        .expect("named metadata");
    assert!(std::ptr::eq(named_first, named_second));
    assert_eq!(NAMED_PROVIDER_CALLS.load(Ordering::SeqCst), named_calls);
    assert_eq!(
        named_first.model_id().map(ModelId::as_str),
        Some("example.CountedNamedProvider")
    );
}

#[test]
fn test_metadata_cache_isolated_between_reflection_contexts() {
    let first_reflection = explicit_model_context::<SnapshotIsolatedFixture>(first_snapshot_metadata);
    let first = ModelRegistry::from_context(&first_reflection).expect("first registry");
    let descriptor = first_reflection
        .type_by_id(std::any::TypeId::of::<SnapshotIsolatedFixture>())
        .expect("type");
    assert!(std::ptr::eq(
        first.metadata_for(&descriptor).unwrap().unwrap(),
        first_snapshot_metadata(),
    ));

    let second_reflection = explicit_model_context::<SnapshotIsolatedFixture>(second_snapshot_metadata);
    let second = ModelRegistry::from_context(&second_reflection).expect("second registry");
    let second_descriptor = second_reflection
        .type_by_id(std::any::TypeId::of::<SnapshotIsolatedFixture>())
        .expect("second context type");
    assert!(std::ptr::eq(
        second.metadata_for(&second_descriptor).unwrap().unwrap(),
        second_snapshot_metadata(),
    ));
    assert_ne!(
        first.metadata_for(&descriptor).unwrap().unwrap().model_id(),
        second.metadata_for(&second_descriptor).unwrap().unwrap().model_id(),
    );
}

/// Consumers can enumerate immutable model metadata and registration
/// provenance.
#[test]
fn test_registry_exposes_read_only_entries_with_sources() {
    let item = entry("example.RegistryEntry", 31);
    let registry = ModelRegistry::from_static_metadata(registry_context(), &[item]).expect("valid registry");
    let entries = registry.entries();
    assert_eq!(entries.len(), 1);
    let entry = registry.get("example.RegistryEntry").expect("entry");
    assert_eq!(entry.model_id().as_str(), "example.RegistryEntry");
    assert!(std::ptr::eq(entry.metadata().expect("concrete metadata"), item.0));
    assert_eq!(entry.source(), item.1);
    assert_eq!(registry.source("example.RegistryEntry"), Some(item.1));
    #[cfg(feature = "generic")]
    assert!(registry.generic("example.RegistryEntry").is_none());
    #[cfg(feature = "generic")]
    assert!(entry.generic_metadata().is_none());
}

/// Static constructors retain local properties while reflection contexts
/// include impl getters.
#[test]
fn test_static_registry_excludes_independently_registered_model_impl_getter() {
    let metadata = TypeMetadata::of::<StaticPropertyFixture>();
    let source = registry_origin("fixture", "tests", line!(), 1, "model", 992);
    let registry =
        ModelRegistry::from_static_metadata(registry_context(), &[(metadata, &source)]).expect("static registry");
    let properties = registry.properties_for(metadata).expect("static properties");
    assert!(properties.property("name").expect("stored property").getter().is_none());

    #[cfg(feature = "generic")]
    {
        let registry =
            ModelRegistry::from_static_metadata_with_generics(registry_context(), &[(metadata, &source)], &[])
                .expect("static registry with generic inputs");
        let properties = registry.properties_for(metadata).expect("static properties");
        assert!(properties.property("name").expect("stored property").getter().is_none());
    }

    let reflection = ReflectionContext::builder()
        .include_type::<StaticPropertyFixture>()
        .register(model_metadata_registration::<StaticPropertyFixture>())
        .register(model_impl_registration::<StaticPropertyFixture>())
        .build()
        .expect("explicit model and impl capabilities");
    let properties = ModelRegistry::from_context(&reflection)
        .expect("context-bound model registry")
        .properties_for(metadata)
        .expect("context-bound properties");
    let getter = properties
        .property("name")
        .expect("stored property")
        .getter()
        .expect("impl getter");
    assert_eq!(getter.rust_method_name(), "name");
    assert_eq!(getter.output_type_id(), std::any::TypeId::of::<str>());
    let setter = properties
        .property("name")
        .expect("stored property")
        .setter()
        .expect("impl setter");
    assert_eq!(setter.rust_method_name(), "set_name");
    assert_eq!(setter.input_type_id(), std::any::TypeId::of::<String>());
}

#[test]
fn test_global_model_registration_includes_aggregated_model_impl_properties() {
    let metadata = TypeMetadata::of::<LinkedModelImplFixture>();
    let properties = ModelRegistry::try_global()
        .expect("global model registry")
        .properties_for(metadata)
        .expect("global properties");
    let property = properties.property("info").expect("info property");
    let getter = property.getter().expect("linked ModelImpl getter");
    assert_eq!(getter.rust_method_name(), "info");
    assert_eq!(getter.output_type_id(), std::any::TypeId::of::<str>());
}
