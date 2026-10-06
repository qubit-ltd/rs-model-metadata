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
use qubit_reflect::TypeDescriptor;
use qubit_reflect::registry::RegistrySnapshotBuilder;

#[Model]
struct NamedFixture {
    value: String,
}

#[Model]
struct MissingPropertyFixture {
    value: String,
}

#[test]
fn test_type_metadata_delegates_structure_to_reflection() {
    let descriptor = TypeDescriptor::of::<NamedFixture>();
    let metadata = TypeMetadata::of::<NamedFixture>();

    assert!(std::ptr::eq(metadata.descriptor(), descriptor));
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
    assert!(std::ptr::eq(
        metadata
            .field("value")
            .expect("named field")
            .reflect()
            .expect("concrete field"),
        descriptor.field_at(0).expect("reflected field"),
    ));
    assert!(metadata.try_properties().is_ok());
}

#[test]
fn test_missing_property_lookup_returns_none_for_static_metadata() {
    let metadata = TypeMetadata::of::<MissingPropertyFixture>();

    assert!(metadata.try_property("missing").unwrap().is_none());
    let registry = RegistrySnapshotBuilder::new()
        .build()
        .expect("empty reflection snapshot");
    assert!(
        metadata
            .try_property_in(&registry, "missing")
            .unwrap()
            .is_none()
    );
}

#[test]
fn test_reflect_only_types_do_not_acquire_model_metadata() {
    let descriptor = TypeDescriptor::of::<String>();
    let registry = ModelRegistry::try_global().expect("model registry must initialize");
    assert!(
        registry
            .metadata_for(descriptor)
            .expect("valid absent capability")
            .is_none()
    );
}
