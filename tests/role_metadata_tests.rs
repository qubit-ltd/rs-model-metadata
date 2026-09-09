// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for the five model-role metadata payloads.

use qubit_model_metadata::__private::v6;
use qubit_model_metadata::metadata::DeclaredEntityTarget;
use qubit_model_metadata::metadata::EnumVariantMetadata;
use qubit_model_metadata::metadata::FieldAttributeMetadata;
use qubit_model_metadata::metadata::FieldMetadata;
use qubit_model_metadata::metadata::IdentifierAssignment;
use qubit_model_metadata::metadata::IdentifierMetadata;
use qubit_model_metadata::metadata::ModelId;
use qubit_model_metadata::metadata::ModelMetadata;
use qubit_model_metadata::metadata::ModelRole;
use qubit_model_metadata::metadata::RoleMetadata;
use qubit_model_metadata::metadata::SerdeFieldMetadata;
use qubit_reflect::Reflect;
use qubit_reflect::TypeDescriptor;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct EntityFixture {
    id: u64,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
enum EnumFixture {
    Ready,
}

#[test]
fn test_five_role_payloads_expose_only_role_specific_facts() {
    let descriptor = TypeDescriptor::of::<EntityFixture>();
    let identifier = Box::leak(Box::new(FieldMetadata::from_reflect(
        descriptor.field_at(0).expect("identifier field"),
    )));
    let RoleMetadata::Entity(entity) = v6::entity_role(identifier) else {
        unreachable!()
    };
    let source = Box::leak(Box::new(DeclaredEntityTarget::ModelId(ModelId::new("example.Source"))));
    let RoleMetadata::Projection(projection) = v6::projection_role(identifier, Some(source)) else {
        unreachable!()
    };
    let RoleMetadata::Projection(open_projection) = v6::projection_role(identifier, None) else {
        unreachable!()
    };
    let model = ModelMetadata;
    let RoleMetadata::Value(value) = v6::value_role(Some(identifier), None) else {
        unreachable!()
    };
    let RoleMetadata::Value(opaque_value) = v6::value_role(None, None) else {
        unreachable!()
    };

    assert!(std::ptr::eq(entity.identifier(), identifier));
    assert!(std::ptr::eq(projection.identifier(), identifier));
    assert_eq!(
        projection
            .source()
            .and_then(DeclaredEntityTarget::model_id)
            .map(|id| id.as_str()),
        Some("example.Source"),
    );
    assert!(!projection.is_open());
    assert!(projection.is_fixed());
    assert!(open_projection.source().is_none());
    assert!(open_projection.is_open());
    assert!(!open_projection.is_fixed());
    assert!(value.is_transparent());
    assert!(std::ptr::eq(
        value.transparent_field().expect("transparent field"),
        identifier
    ));
    assert!(value.canonical_codec().is_none());
    assert!(!opaque_value.is_transparent());
    assert!(opaque_value.transparent_field().is_none());
    assert!(matches!(RoleMetadata::Entity(entity).role(), ModelRole::Entity));
    assert!(matches!(
        RoleMetadata::Projection(projection).role(),
        ModelRole::Projection
    ));
    assert!(matches!(RoleMetadata::Model(model).role(), ModelRole::Model));
    assert!(matches!(RoleMetadata::Value(value).role(), ModelRole::Value));
}

#[test]
fn test_enum_variant_keeps_rust_canonical_and_directional_serde_names() {
    let descriptor = TypeDescriptor::of::<EnumFixture>();
    let reflect = descriptor.variants().first().expect("enum variant");
    let variant = v6::enum_variant_metadata(reflect, "READY", "ready-out", "ready-in", &[], true);
    let variants = Box::leak(vec![variant].into_boxed_slice());
    let RoleMetadata::Enum(metadata) = v6::enum_role(variants) else {
        unreachable!()
    };

    assert_eq!(
        metadata.variant("READY").map(EnumVariantMetadata::rust_name),
        Some("Ready")
    );
    assert_eq!(
        metadata
            .variant_by_rust_name("Ready")
            .map(EnumVariantMetadata::serialized_name),
        Some("ready-out"),
    );
    assert_eq!(
        metadata
            .variant_by_serialized_name("ready-out")
            .map(EnumVariantMetadata::deserialized_name),
        Some("ready-in"),
    );
    assert!(metadata.variants()[0].is_default());
    assert!(matches!(RoleMetadata::Enum(metadata).role(), ModelRole::Enum));
}

#[test]
fn test_type_metadata_navigates_fields_and_role_without_copying_reflection_facts() {
    let descriptor = TypeDescriptor::of::<EntityFixture>();
    let identifier = Box::leak(Box::new(IdentifierMetadata::new(IdentifierAssignment::Application)));
    let attributes = Box::leak(vec![FieldAttributeMetadata::Identifier(identifier)].into_boxed_slice());
    let fields = Box::leak(
        vec![v6::field_metadata(
            descriptor.field_at(0).expect("identifier field"),
            attributes,
            &[],
            &[],
            &SerdeFieldMetadata::DEFAULT,
        )]
        .into_boxed_slice(),
    );
    let role = Box::leak(Box::new(v6::entity_role(&fields[0])));
    let metadata =
        v6::GeneratedTypeMetadataBuilder::new(descriptor, Some(ModelId::new("example.Entity")), fields, role)
            .finish::<EntityFixture>();

    assert!(std::ptr::eq(metadata.descriptor(), descriptor));
    assert_eq!(metadata.type_id(), descriptor.type_id());
    assert_eq!(metadata.type_name(), descriptor.type_name());
    assert_eq!(metadata.model_id().map(|id| id.as_str()), Some("example.Entity"));
    assert!(metadata.is_registered());
    assert_eq!(metadata.role(), ModelRole::Entity);
    assert!(std::ptr::eq(metadata.fields(), fields));
    assert!(std::ptr::eq(metadata.field("id").expect("named field"), &fields[0]));
    assert!(std::ptr::eq(metadata.field_at(0).expect("indexed field"), &fields[0]));
    assert!(metadata.as_entity().is_some());
    assert!(metadata.as_projection().is_none());
    assert!(metadata.as_model().is_none());
    assert!(metadata.as_enum().is_none());
    assert!(metadata.as_value().is_none());
    #[cfg(feature = "generic")]
    {
        assert!(metadata.generic_definition().is_none());
        assert!(metadata.concrete_generic().is_none());
    }
    assert!(matches!(metadata.role_metadata(), RoleMetadata::Entity(_)));
    assert!(metadata.validate_for::<EntityFixture>().is_ok());
    metadata.assert_valid_for::<EntityFixture>();
}
