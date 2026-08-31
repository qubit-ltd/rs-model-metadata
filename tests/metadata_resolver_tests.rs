// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

// qubit-style: allow explicit-imports
//! Integration tests for explicit model graph resolution.

use qubit_model_metadata::DeclaredEntityTarget;
use qubit_model_metadata::EntityMetadata;
use qubit_model_metadata::FieldAttributeMetadata;
use qubit_model_metadata::FieldDescriptor;
use qubit_model_metadata::FieldMetadata;
use qubit_model_metadata::FieldReferenceMetadata;
use qubit_model_metadata::IdentifierAssignment;
use qubit_model_metadata::IdentifierMetadata;
use qubit_model_metadata::IndexingReasons;
use qubit_model_metadata::ModelId;
use qubit_model_metadata::ModelMetadata;
use qubit_model_metadata::ModelRegistration;
use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::ModelResolveErrorKind;
use qubit_model_metadata::ModelResolver;
use qubit_model_metadata::PropertyMetadata;
use qubit_model_metadata::PropertyPath;
use qubit_model_metadata::ReferenceSelection;
use qubit_model_metadata::Reflect;
use qubit_model_metadata::ResolveInputs;
use qubit_model_metadata::RoleMetadata;
use qubit_model_metadata::SerdeFieldMetadata;
use qubit_model_metadata::TypeDescriptor;
use qubit_model_metadata::TypeMetadata;
use qubit_model_metadata::ValueMetadata;
use qubit_model_metadata::identity::FragmentIdentity;

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
        vec![FieldMetadata::with_semantics(
            target_descriptor.field_at(0).unwrap(),
            target_attributes,
            &[],
            &[],
            &SerdeFieldMetadata::DEFAULT,
        )]
        .into_boxed_slice(),
    );
    let target_role = Box::leak(Box::new(RoleMetadata::Entity(EntityMetadata::new(&target_fields[0]))));
    let target_properties = Box::leak(
        vec![PropertyMetadata::new(
            "id",
            target_fields[0].type_ref(),
            Some(&target_fields[0]),
            None,
            None,
        )]
        .into_boxed_slice(),
    );
    let target_metadata = Box::leak(Box::new(
        TypeMetadata::new(
            target_descriptor,
            Some(ModelId::new("example.Target")),
            target_fields,
            target_role,
        )
        .with_properties(target_properties),
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
        vec![FieldMetadata::with_semantics(
            source_descriptor.field_at(0).unwrap(),
            attributes,
            &[],
            &[],
            &SerdeFieldMetadata::DEFAULT,
        )]
        .into_boxed_slice(),
    );
    let source_role = Box::leak(Box::new(RoleMetadata::Model(ModelMetadata)));
    let source_metadata = Box::leak(Box::new(TypeMetadata::new(
        source_descriptor,
        Some(ModelId::new("example.Source")),
        source_fields,
        source_role,
    )));

    let target_registration: &'static ModelRegistration = Box::leak(Box::new(ModelRegistration::from_concrete(
        target_metadata,
        source_identity(1),
    )));
    let source_registration: &'static ModelRegistration = Box::leak(Box::new(ModelRegistration::from_concrete(
        source_metadata,
        source_identity(2),
    )));
    let registry = ModelRegistry::from_registrations([target_registration, source_registration]).unwrap();
    let graph = ModelResolver::new(ResolveInputs { models: &registry })
        .resolve_all()
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
        vec![FieldMetadata::with_semantics(
            descriptor.field_at(0).unwrap(),
            attributes,
            &[],
            &[],
            &SerdeFieldMetadata::DEFAULT,
        )]
        .into_boxed_slice(),
    );
    let role = Box::leak(Box::new(RoleMetadata::Model(ModelMetadata)));
    let metadata = Box::leak(Box::new(TypeMetadata::new(
        descriptor,
        Some(ModelId::new("example.SourceMissing")),
        fields,
        role,
    )));
    let registration: &'static ModelRegistration =
        Box::leak(Box::new(ModelRegistration::from_concrete(metadata, source_identity(3))));
    let registry = ModelRegistry::from_registrations([registration]).unwrap();
    let errors = ModelResolver::new(ResolveInputs { models: &registry })
        .resolve_all()
        .expect_err("missing targets must prevent graph publication");

    assert_eq!(errors.errors().len(), 1);
    assert_eq!(errors.errors()[0].kind(), ModelResolveErrorKind::MissingModelId);
    assert_eq!(errors.errors()[0].model_id(), Some("missing.Target"));
}

fn indexed_field(reflect: &'static FieldDescriptor) -> FieldMetadata {
    let attributes = Box::leak(vec![FieldAttributeMetadata::Indexed(IndexingReasons::EXPLICIT)].into_boxed_slice());
    FieldMetadata::with_semantics(reflect, attributes, &[], &[], &SerdeFieldMetadata::DEFAULT)
}

fn entity_metadata(
    descriptor: &'static TypeDescriptor,
    id: &'static str,
    fields: &'static [FieldMetadata],
) -> &'static TypeMetadata {
    let role = Box::leak(Box::new(RoleMetadata::Entity(EntityMetadata::new(&fields[0]))));
    Box::leak(Box::new(TypeMetadata::new(
        descriptor,
        Some(ModelId::new(id)),
        fields,
        role,
    )))
}

#[test]
fn test_query_recurses_indexed_value_fields_and_reports_flat_name_conflicts() {
    let nested_descriptor = TypeDescriptor::of::<NestedQueryFixture>();
    let nested_fields = Box::leak(vec![indexed_field(nested_descriptor.field_at(0).unwrap())].into_boxed_slice());
    let nested_role = Box::leak(Box::new(RoleMetadata::Value(ValueMetadata::new(None, None))));
    let nested = Box::leak(Box::new(TypeMetadata::new(
        nested_descriptor,
        Some(ModelId::new("query.Nested")),
        nested_fields,
        nested_role,
    )));

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
            FieldMetadata::with_semantics(
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
    let root = entity_metadata(root_descriptor, "query.Root", root_fields);
    let nested_registration: &'static ModelRegistration =
        Box::leak(Box::new(ModelRegistration::from_concrete(nested, source_identity(10))));
    let root_registration: &'static ModelRegistration =
        Box::leak(Box::new(ModelRegistration::from_concrete(root, source_identity(11))));
    let registry = ModelRegistry::from_registrations([nested_registration, root_registration]).unwrap();
    let graph = ModelResolver::new(ResolveInputs { models: &registry })
        .resolve_all()
        .unwrap();
    let query = graph.query(root.as_entity().unwrap()).unwrap();
    assert_eq!(
        query.filter_by_flat_name("nested_b").unwrap().path().segments(),
        &["nested", "b"]
    );

    let conflict_descriptor = TypeDescriptor::of::<ConflictingQueryFixture>();
    let conflict_fields = Box::leak(
        vec![
            FieldMetadata::with_semantics(
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
    let conflict = entity_metadata(conflict_descriptor, "query.Conflict", conflict_fields);
    let conflict_registration: &'static ModelRegistration = Box::leak(Box::new(ModelRegistration::from_concrete(
        conflict,
        source_identity(12),
    )));
    let registry = ModelRegistry::from_registrations([nested_registration, conflict_registration]).unwrap();
    let errors = ModelResolver::new(ResolveInputs { models: &registry })
        .resolve_all()
        .unwrap_err();
    assert!(
        errors
            .errors()
            .iter()
            .any(|error| error.kind() == ModelResolveErrorKind::QueryNameConflict)
    );
}

#[test]
fn test_resolver_rejects_value_closure_over_model_role() {
    let model_descriptor = TypeDescriptor::of::<PlainModelFixture>();
    let model_fields =
        Box::leak(vec![FieldMetadata::from_reflect(model_descriptor.field_at(0).unwrap())].into_boxed_slice());
    let model_role = Box::leak(Box::new(RoleMetadata::Model(ModelMetadata)));
    let model = Box::leak(Box::new(TypeMetadata::new(
        model_descriptor,
        Some(ModelId::new("closure.Model")),
        model_fields,
        model_role,
    )));
    let value_descriptor = TypeDescriptor::of::<InvalidValueFixture>();
    let value_fields =
        Box::leak(vec![FieldMetadata::from_reflect(value_descriptor.field_at(0).unwrap())].into_boxed_slice());
    let value_role = Box::leak(Box::new(RoleMetadata::Value(ValueMetadata::new(None, None))));
    let value = Box::leak(Box::new(TypeMetadata::new(
        value_descriptor,
        Some(ModelId::new("closure.Value")),
        value_fields,
        value_role,
    )));
    let model_registration: &'static ModelRegistration =
        Box::leak(Box::new(ModelRegistration::from_concrete(model, source_identity(20))));
    let value_registration: &'static ModelRegistration =
        Box::leak(Box::new(ModelRegistration::from_concrete(value, source_identity(21))));
    let registry = ModelRegistry::from_registrations([model_registration, value_registration]).unwrap();
    let errors = ModelResolver::new(ResolveInputs { models: &registry })
        .resolve_all()
        .unwrap_err();
    assert!(
        errors
            .errors()
            .iter()
            .any(|error| error.kind() == ModelResolveErrorKind::InvalidValueClosure)
    );
}
