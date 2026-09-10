// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for role-aware type metadata.

use qubit_model_metadata::__private::v7;
use qubit_model_metadata::metadata::FieldMetadata;
use qubit_model_metadata::metadata::ModelRole;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_reflect::Reflect;
use qubit_reflect::TypeDescriptor;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct NamedFixture {
    value: String,
}

#[test]
fn test_type_metadata_delegates_structure_to_reflection() {
    let descriptor = TypeDescriptor::of::<NamedFixture>();
    let fields = Box::leak(
        descriptor
            .fields()
            .iter()
            .map(|field| FieldMetadata::from_reflect(field.declaring_type().type_id(), field))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    let role = v7::leak(v7::model_role());
    let metadata: &'static TypeMetadata =
        v7::leak(v7::GeneratedTypeMetadataBuilder::new(descriptor, None, fields, role).finish::<NamedFixture>());

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
