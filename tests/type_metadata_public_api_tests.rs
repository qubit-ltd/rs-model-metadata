// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for role-aware type metadata.

use qubit_model_derive::Model;
use qubit_model_metadata::metadata::ModelRole;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_reflect::ReflectionContext;

#[Model]
struct NamedFixture {
    value: String,
}

#[Model]
struct MissingPropertyFixture {
    value: String,
}

#[Model(id = "type_metadata_public_api.registered")]
struct RegisteredMetadataFixture {
    value: String,
}

#[test]
fn test_type_metadata_delegates_structure_to_reflection() {
    let context = ReflectionContext::builder()
        .include_type::<NamedFixture>()
        .include_type::<String>()
        .build()
        .expect("reflection context");
    let descriptor = context.type_of::<NamedFixture>();
    let metadata = TypeMetadata::of::<NamedFixture>();
    let bound = metadata.descriptor_in(&context);

    assert_eq!(metadata.type_id(), descriptor.type_id());
    assert!(std::ptr::eq(bound.context(), &context));
    assert_eq!(metadata.role(), ModelRole::Model);
    assert!(metadata.as_model().is_some());
    assert!(metadata.as_entity().is_none());
    assert!(metadata.as_projection().is_none());
    assert!(metadata.as_enum().is_none());
    assert!(metadata.as_value().is_none());
    assert_eq!(metadata.model_id(), None);
    assert!(!metadata.is_registered());
    metadata.validate_for::<NamedFixture>().unwrap();
    metadata.assert_valid_for::<NamedFixture>();
    assert_eq!(metadata.fields().len(), descriptor.fields().len());
    let field = metadata.field("value").expect("named field");
    let reflected = field.reflect_in(&context).expect("concrete field");
    assert_eq!(reflected.index(), 0);
    assert!(std::ptr::eq(reflected.context(), &context));
    assert!(metadata.try_properties().is_ok());
    assert!(metadata.field_at(0).is_some());
    assert!(metadata.field_at(1).is_none());
    assert!(metadata.try_property("value").unwrap().is_some());
    assert!(metadata.try_property_in(&context, "value").unwrap().is_some());
    assert!(metadata.try_properties_in(&context).is_ok());
    assert!(metadata.property_fragments().is_ok());
    assert!(metadata.property_fragments_in(&context).is_ok());
}

#[test]
fn test_registered_type_metadata_exposes_registration_identity() {
    let metadata = TypeMetadata::of::<RegisteredMetadataFixture>();
    let context = ReflectionContext::builder()
        .include_type::<RegisteredMetadataFixture>()
        .include_type::<String>()
        .build()
        .expect("registered model context");

    assert!(metadata.is_registered());
    assert_eq!(
        metadata.model_id().unwrap().as_str(),
        "type_metadata_public_api.registered"
    );
    assert!(metadata.try_property("value").unwrap().is_some());
    assert!(metadata.try_property_in(&context, "value").unwrap().is_some());
}

#[test]
fn test_type_metadata_validation_rejects_a_different_rust_root() {
    let metadata = TypeMetadata::of::<NamedFixture>();

    assert!(metadata.validate_for::<String>().is_err());
}

#[test]
fn test_missing_property_lookup_returns_none_for_static_metadata() {
    let metadata = TypeMetadata::of::<MissingPropertyFixture>();

    assert!(metadata.try_property("missing").unwrap().is_none());
    let context = ReflectionContext::builder()
        .include_type::<MissingPropertyFixture>()
        .include_type::<String>()
        .build()
        .expect("explicit context");
    assert!(metadata.try_property_in(&context, "missing").unwrap().is_none());
}

#[test]
fn test_reflect_only_types_do_not_acquire_model_metadata() {
    let context = ReflectionContext::builder()
        .include_type::<String>()
        .build()
        .expect("string context");
    let descriptor = context.type_of::<String>();
    let registry = ModelRegistry::from_context(&context).expect("model registry must initialize");
    assert!(
        registry
            .metadata_for(&descriptor)
            .expect("valid absent capability")
            .is_none()
    );
}
