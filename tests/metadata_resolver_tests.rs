// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow explicit-imports
//! Integration tests for explicit model graph resolution.

use qubit_model_metadata::__private::v5;
use qubit_model_metadata::metadata::DeclaredEntityTarget;
use qubit_model_metadata::metadata::FieldAttributeMetadata;
use qubit_model_metadata::metadata::FieldMetadata;
use qubit_model_metadata::metadata::FieldReferenceMetadata;
use qubit_model_metadata::metadata::IdentifierAssignment;
use qubit_model_metadata::metadata::IdentifierMetadata;
use qubit_model_metadata::metadata::IndexingReasons;
use qubit_model_metadata::metadata::ModelId;
use qubit_model_metadata::metadata::PropertyMetadata;
use qubit_model_metadata::metadata::PropertyPath;
use qubit_model_metadata::metadata::ReferenceSelection;
use qubit_model_metadata::metadata::SerdeFieldMetadata;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveErrorKind;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_reflect::FieldDescriptor;
use qubit_reflect::Reflect;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::identity::FragmentIdentity;

fn model_registry(entries: &[(&'static TypeMetadata, &'static FragmentIdentity)]) -> ModelRegistry<'static> {
    ModelRegistry::from_metadata(entries).expect("valid isolated model registry")
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct TargetFixture {
    id: u64,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct SourceFixture {
    target_id: u64,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct NestedQueryFixture {
    b: u64,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct RootQueryFixture {
    id: u64,
    nested: NestedQueryFixture,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct ConflictingQueryFixture {
    id: u64,
    a_b: u64,
    a: NestedQueryFixture,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct PlainModelFixture {
    value: u64,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct InvalidValueFixture {
    model: PlainModelFixture,
}

fn source_identity(line: u32) -> &'static FragmentIdentity {
    Box::leak(Box::new(FragmentIdentity::new(
        "fixture",
        "tests",
        line,
        1,
        "model",
        line as u64,
    )))
}

#[test]
fn test_resolver_resolves_reference_targets_and_properties() {
    let target_descriptor = TypeDescriptor::of::<TargetFixture>();
    let identifier = Box::leak(Box::new(IdentifierMetadata::new(IdentifierAssignment::Application)));
    let target_attributes = Box::leak(
        vec![
            FieldAttributeMetadata::Identifier(identifier),
            FieldAttributeMetadata::Indexed(IndexingReasons::IDENTIFIER),
        ]
        .into_boxed_slice(),
    );
    let target_fields = Box::leak(
        vec![v5::field_metadata(
            target_descriptor.field_at(0).unwrap(),
            target_attributes,
            &[],
            &[],
            &SerdeFieldMetadata::DEFAULT,
        )]
        .into_boxed_slice(),
    );
    let target_role = v5::leak(v5::entity_role(&target_fields[0]));
    let target_properties = Box::leak(
        vec![v5::property_metadata(
            "id",
            target_fields[0].type_ref(),
            Some(&target_fields[0]),
            None,
            None,
        )]
        .into_boxed_slice(),
    );
    let target_metadata = Box::leak(Box::new(
        v5::GeneratedTypeMetadataBuilder::new(
            target_descriptor,
            Some(ModelId::new("example.Target")),
            target_fields,
            target_role,
        )
        .properties(target_properties)
        .finish::<TargetFixture>(),
    ));

    let declared_target = Box::leak(Box::new(DeclaredEntityTarget::ModelId(ModelId::new("example.Target"))));
    let selection = Box::leak(Box::new(ReferenceSelection::Property(PropertyPath::new(&["id"]))));
    let reference = Box::leak(Box::new(FieldReferenceMetadata::new(
        declared_target,
        selection,
        true,
        None,
    )));
    let attributes = Box::leak(vec![FieldAttributeMetadata::Reference(reference)].into_boxed_slice());
    let source_descriptor = TypeDescriptor::of::<SourceFixture>();
    let source_fields = Box::leak(
        vec![v5::field_metadata(
            source_descriptor.field_at(0).unwrap(),
            attributes,
            &[],
            &[],
            &SerdeFieldMetadata::DEFAULT,
        )]
        .into_boxed_slice(),
    );
    let source_role = v5::leak(v5::model_role());
    let source_metadata = v5::leak(
        v5::GeneratedTypeMetadataBuilder::new(
            source_descriptor,
            Some(ModelId::new("example.Source")),
            source_fields,
            source_role,
        )
        .finish::<SourceFixture>(),
    );

    let registry = model_registry(&[
        (target_metadata, source_identity(1)),
        (source_metadata, source_identity(2)),
    ]);
    let graph = StructureResolver::new(ResolveInputs { models: &registry })
        .resolve()
        .unwrap();
    let resolved = graph.reference(&source_fields[0]).expect("resolved reference");

    assert!(std::ptr::eq(resolved.target(), target_metadata));
    assert_eq!(resolved.property().map(PropertyMetadata::name), Some("id"));
    let query = graph.query(target_metadata.as_entity().unwrap()).expect("entity query");
    assert!(query.filters().is_empty());
    assert_eq!(query.unique_keys().len(), 1);
}

#[test]
fn test_resolver_aggregates_missing_targets_deterministically() {
    let descriptor = TypeDescriptor::of::<SourceFixture>();
    let target = Box::leak(Box::new(DeclaredEntityTarget::ModelId(ModelId::new("missing.Target"))));
    let selection = Box::leak(Box::new(ReferenceSelection::Entity));
    let reference = Box::leak(Box::new(FieldReferenceMetadata::new(target, selection, true, None)));
    let attributes = Box::leak(vec![FieldAttributeMetadata::Reference(reference)].into_boxed_slice());
    let fields = Box::leak(
        vec![v5::field_metadata(
            descriptor.field_at(0).unwrap(),
            attributes,
            &[],
            &[],
            &SerdeFieldMetadata::DEFAULT,
        )]
        .into_boxed_slice(),
    );
    let role = v5::leak(v5::model_role());
    let metadata = v5::leak(
        v5::GeneratedTypeMetadataBuilder::new(descriptor, Some(ModelId::new("example.SourceMissing")), fields, role)
            .finish::<SourceFixture>(),
    );
    let registry = model_registry(&[(metadata, source_identity(3))]);
    let errors = StructureResolver::new(ResolveInputs { models: &registry })
        .resolve()
        .expect_err("missing targets must prevent graph publication");

    assert_eq!(errors.errors().len(), 1);
    assert_eq!(errors.errors()[0].kind(), ResolveErrorKind::MissingModelId);
    assert_eq!(errors.errors()[0].model_id(), Some("missing.Target"));
}

fn indexed_field(reflect: &'static FieldDescriptor) -> FieldMetadata {
    let attributes = Box::leak(vec![FieldAttributeMetadata::Indexed(IndexingReasons::EXPLICIT)].into_boxed_slice());
    v5::field_metadata(reflect, attributes, &[], &[], &SerdeFieldMetadata::DEFAULT)
}

fn entity_metadata<T: 'static>(
    descriptor: &'static TypeDescriptor,
    id: &'static str,
    fields: &'static [FieldMetadata],
) -> &'static TypeMetadata {
    let role = v5::leak(v5::entity_role(&fields[0]));
    v5::leak(v5::GeneratedTypeMetadataBuilder::new(descriptor, Some(ModelId::new(id)), fields, role).finish::<T>())
}

#[test]
fn test_query_recurses_indexed_value_fields_and_reports_flat_name_conflicts() {
    let nested_descriptor = TypeDescriptor::of::<NestedQueryFixture>();
    let nested_fields = Box::leak(vec![indexed_field(nested_descriptor.field_at(0).unwrap())].into_boxed_slice());
    let nested_role = v5::leak(v5::value_role(None, None));
    let nested = v5::leak(
        v5::GeneratedTypeMetadataBuilder::new(
            nested_descriptor,
            Some(ModelId::new("query.Nested")),
            nested_fields,
            nested_role,
        )
        .finish::<NestedQueryFixture>(),
    );

    let root_descriptor = TypeDescriptor::of::<RootQueryFixture>();
    let identifier = Box::leak(Box::new(IdentifierMetadata::new(IdentifierAssignment::Application)));
    let identifier_attributes = Box::leak(
        vec![
            FieldAttributeMetadata::Identifier(identifier),
            FieldAttributeMetadata::Indexed(IndexingReasons::IDENTIFIER),
        ]
        .into_boxed_slice(),
    );
    let root_fields = Box::leak(
        vec![
            v5::field_metadata(
                root_descriptor.field_at(0).unwrap(),
                identifier_attributes,
                &[],
                &[],
                &SerdeFieldMetadata::DEFAULT,
            ),
            indexed_field(root_descriptor.field_at(1).unwrap()),
        ]
        .into_boxed_slice(),
    );
    let root = entity_metadata::<RootQueryFixture>(root_descriptor, "query.Root", root_fields);
    let registry = model_registry(&[(nested, source_identity(10)), (root, source_identity(11))]);
    let graph = StructureResolver::new(ResolveInputs { models: &registry })
        .resolve()
        .unwrap();
    let query = graph.query(root.as_entity().unwrap()).unwrap();
    assert_eq!(
        query.filter_by_flat_name("nested_b").unwrap().path().segments(),
        &["nested", "b"]
    );

    let conflict_descriptor = TypeDescriptor::of::<ConflictingQueryFixture>();
    let conflict_fields = Box::leak(
        vec![
            v5::field_metadata(
                conflict_descriptor.field_at(0).unwrap(),
                identifier_attributes,
                &[],
                &[],
                &SerdeFieldMetadata::DEFAULT,
            ),
            indexed_field(conflict_descriptor.field_at(1).unwrap()),
            indexed_field(conflict_descriptor.field_at(2).unwrap()),
        ]
        .into_boxed_slice(),
    );
    let conflict = entity_metadata::<ConflictingQueryFixture>(conflict_descriptor, "query.Conflict", conflict_fields);
    let registry = model_registry(&[(nested, source_identity(10)), (conflict, source_identity(12))]);
    let errors = StructureResolver::new(ResolveInputs { models: &registry })
        .resolve()
        .unwrap_err();
    assert!(
        errors
            .errors()
            .iter()
            .any(|error| error.kind() == ResolveErrorKind::QueryNameConflict)
    );
}

#[test]
fn test_resolver_rejects_value_closure_over_model_role() {
    let model_descriptor = TypeDescriptor::of::<PlainModelFixture>();
    let model_fields =
        Box::leak(vec![FieldMetadata::from_reflect(model_descriptor.field_at(0).unwrap())].into_boxed_slice());
    let model_role = v5::leak(v5::model_role());
    let model = v5::leak(
        v5::GeneratedTypeMetadataBuilder::new(
            model_descriptor,
            Some(ModelId::new("closure.Model")),
            model_fields,
            model_role,
        )
        .finish::<PlainModelFixture>(),
    );
    let value_descriptor = TypeDescriptor::of::<InvalidValueFixture>();
    let value_fields =
        Box::leak(vec![FieldMetadata::from_reflect(value_descriptor.field_at(0).unwrap())].into_boxed_slice());
    let value_role = v5::leak(v5::value_role(None, None));
    let value = v5::leak(
        v5::GeneratedTypeMetadataBuilder::new(
            value_descriptor,
            Some(ModelId::new("closure.Value")),
            value_fields,
            value_role,
        )
        .finish::<InvalidValueFixture>(),
    );
    let registry = model_registry(&[(model, source_identity(20)), (value, source_identity(21))]);
    let errors = StructureResolver::new(ResolveInputs { models: &registry })
        .resolve()
        .unwrap_err();
    assert!(
        errors
            .errors()
            .iter()
            .any(|error| error.kind() == ResolveErrorKind::InvalidValueClosure)
    );
}
