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
use qubit_model_metadata::__private::v3;
use qubit_model_metadata::__private::v3::register_model_capability;
use qubit_model_metadata::ModelId;
use qubit_model_metadata::ModelRegistration;
use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::ModelRegistryErrorKind;
use qubit_model_metadata::ModelRole;
use qubit_model_metadata::Reflect;
use qubit_model_metadata::TypeDescriptor;
use qubit_model_metadata::TypeMetadata;
use qubit_model_metadata::identity::FragmentIdentity;
use qubit_model_metadata::registry::ReflectRegistry;

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
            let role = v3::leak(v3::model_role());
            v3::GeneratedTypeMetadataBuilder::new(
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

fn registration(id: &'static str, fingerprint: u64) -> &'static ModelRegistration {
    let role = v3::leak(v3::model_role());
    let metadata = v3::leak(
        v3::GeneratedTypeMetadataBuilder::new(
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
    v3::leak(v3::concrete_registration(metadata, source))
}

#[test]
fn test_registry_indexes_registration_metadata_and_type_identity() {
    let item = registration("example.RegistryFixture", 1);
    let registry = ModelRegistry::from_registrations([item]).expect("valid registry");

    assert!(std::ptr::eq(
        registry.get("example.RegistryFixture").expect("registration"),
        registry.registrations().first().unwrap()
    ));
    assert!(std::ptr::eq(
        registry.metadata("example.RegistryFixture").expect("metadata"),
        item.metadata().unwrap()
    ));
    assert!(std::ptr::eq(
        registry
            .by_type_id(TypeDescriptor::of::<RegistryFixture>().type_id())
            .expect("type lookup"),
        item.metadata().unwrap(),
    ));
    assert!(registry.get("not-valid!").is_none());
}

#[test]
fn test_registry_reports_duplicate_ids_with_both_sources() {
    let first = registration("example.Duplicate", 1);
    let second = registration("example.Duplicate", 2);
    let error = ModelRegistry::from_registrations([second, first]).expect_err("duplicate IDs must fail");

    assert_eq!(error.kind(), ModelRegistryErrorKind::DuplicateModelId);
    assert_eq!(error.model_id().map(|id| id.as_str()), Some("example.Duplicate"));
    assert_eq!(error.sources().len(), 2);
}

#[test]
fn test_registry_indexes_one_generic_definition_without_concrete_model_id() {
    let concrete = TypeDescriptor::of::<GenericFixture<u8>>();
    let definition = concrete.concrete_generic().expect("generic substitutions").definition();
    let generic = v3::leak(v3::generic_model_metadata(
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
    let registration: &'static ModelRegistration = v3::leak(v3::generic_registration(generic, source));
    let registry = ModelRegistry::from_registrations([registration]).expect("generic registry");

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
    let registry = ModelRegistry::from_reflect_registry(reflection, []).expect("valid model projection");
    let registration = registry
        .get("example.ProjectedFixture")
        .expect("projected concrete registration");
    let reflected_source = reflection
        .type_source(TypeDescriptor::of::<ProjectedFixture>().type_id())
        .expect("reflected type source");

    assert!(std::ptr::eq(registration.source(), reflected_source));
    assert!(std::ptr::eq(
        registration.metadata().expect("concrete metadata"),
        ProjectedFixture::__type_metadata(),
    ));
}
