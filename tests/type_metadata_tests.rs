// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Integration tests for role-aware type metadata.

use qubit_model_metadata::FieldMetadata;
use qubit_model_metadata::ModelDescriptorExt;
use qubit_model_metadata::ModelMetadata;
use qubit_model_metadata::ModelRole;
use qubit_model_metadata::Reflect;
use qubit_model_metadata::RoleMetadata;
use qubit_model_metadata::TypeDescriptor;
use qubit_model_metadata::TypeMetadata;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct NamedFixture {
    value: String,
}

static MODEL_ROLE: RoleMetadata = RoleMetadata::Model(ModelMetadata);

#[test]
fn test_type_metadata_delegates_structure_to_reflection() {
    let descriptor = TypeDescriptor::of::<NamedFixture>();
    let fields = Box::leak(
        descriptor
            .fields()
            .iter()
            .map(FieldMetadata::from_reflect)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    let metadata = TypeMetadata::new(descriptor, None, fields, &MODEL_ROLE);

    assert!(std::ptr::eq(metadata.descriptor(), descriptor));
    assert_eq!(metadata.role(), ModelRole::Model);
    assert_eq!(metadata.model_id(), None);
    assert!(!metadata.is_registered());
    assert_eq!(metadata.fields().len(), descriptor.fields().len());
    assert!(std::ptr::eq(
        metadata.field("value").expect("named field").reflect(),
        descriptor.field_at(0).expect("reflected field"),
    ));
}

#[test]
fn test_reflect_only_types_do_not_acquire_model_metadata() {
    let descriptor = TypeDescriptor::of::<String>();
    assert!(descriptor.model_metadata().is_none());
    assert!(!descriptor.is_model_type());
}
