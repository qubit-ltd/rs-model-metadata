// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Graph lookup must preserve declaration identity across metadata copies.

use std::any::TypeId;

use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::Projection;
use qubit_model_metadata::metadata::IndexingReasons;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ContextRequirement;
use qubit_model_metadata::resolve::ResolveErrorKind;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;

#[Entity(id = "identity.Target")]
struct Target {
    #[identifier]
    id: Id,
}

#[Model(id = "identity.Root")]
struct Root {
    #[reference(entity = Target, property = id)]
    target: Id,
}

#[test]
fn test_reference_lookup_preserves_copied_field_identity() {
    let models = ModelRegistry::try_global().expect("valid registry");
    let graph = StructureResolver::new(ResolveInputs { models, roots: &[] })
        .resolve()
        .expect("valid graph");
    let field = TypeMetadata::of::<Root>().field("target").expect("reference field");
    let copy = *field;
    assert!(
        graph.reference(field.location().unwrap()).is_some(),
        "original declaration must resolve"
    );
    assert!(
        graph.reference(copy.location().unwrap()).is_some(),
        "copy must retain declaration identity"
    );
    let reference = graph.reference(copy.location().unwrap()).expect("resolved reference");
    assert_eq!(reference.context_requirement(), ContextRequirement::None);
    assert!(std::ptr::eq(
        reference.declaration(),
        field.reference().expect("original declaration")
    ));
    assert!(std::ptr::eq(graph.registry(), models));
}

#[Model]
struct ParentBoundReference {
    #[reference(entity = Target, property = id, path = "..")]
    target: Id,
}

#[Model]
struct WholeEntityReference {
    #[reference(entity = Target)]
    target: Target,
}

#[Model]
struct WrongWholeEntityReference {
    #[reference(entity = Target)]
    target: Id,
}

/// Structural resolution retains deferred parent navigation and whole-entity
/// selection.
#[test]
fn test_reference_context_and_selection_are_independent() {
    let models = ModelRegistry::try_global().expect("valid registry");
    let parent = TypeMetadata::of::<ParentBoundReference>();
    let whole = TypeMetadata::of::<WholeEntityReference>();
    let roots = [parent, whole];
    let graph = StructureResolver::new(ResolveInputs { models, roots: &roots })
        .resolve()
        .expect("valid references");
    let parent_reference = graph
        .reference(parent.fields()[0].location().expect("parent field"))
        .expect("parent reference");
    assert_eq!(parent_reference.context_requirement(), ContextRequirement::ParentObject);
    assert_eq!(parent_reference.property().expect("selected identifier").name(), "id");
    assert!(
        parent_reference
            .declaration()
            .path()
            .expect("deferred path")
            .requires_parent()
    );
    let whole_reference = graph
        .reference(whole.fields()[0].location().expect("whole field"))
        .expect("entity reference");
    assert_eq!(whole_reference.context_requirement(), ContextRequirement::None);
    assert!(whole_reference.property().is_none());
    assert_eq!(whole_reference.target().type_id(), TypeId::of::<Target>());

    let wrong = TypeMetadata::of::<WrongWholeEntityReference>();
    let roots = [wrong];
    let errors = StructureResolver::new(ResolveInputs { models, roots: &roots })
        .resolve()
        .expect_err("Id cannot hold the complete Entity");
    let error = errors
        .errors()
        .iter()
        .find(|error| error.owner_type_id() == Some(wrong.type_id()))
        .expect("wrong entity value diagnostic");
    assert_eq!(error.kind(), ResolveErrorKind::TypeMismatch);
    assert_eq!(error.expected_type(), Some(TypeId::of::<Target>()));
    assert_eq!(error.actual_type(), Some(TypeId::of::<Id>()));
}

#[Model]
struct Anonymous {
    value: String,
}

#[Enum]
enum Choice {
    Named { name: String },
    Tuple(String),
}

#[Projection(source = Target)]
struct TargetView {
    #[identifier]
    id: Id,
}

#[test]
fn test_anonymous_roots_and_role_queries_use_concrete_type_identity() {
    let models = ModelRegistry::try_global().expect("valid registry");
    let roots = [TypeMetadata::of::<Anonymous>(), TypeMetadata::of::<TargetView>()];
    let graph = StructureResolver::new(ResolveInputs { models, roots: &roots })
        .resolve()
        .expect("valid graph");
    assert_eq!(
        graph.model(TypeId::of::<Anonymous>()).unwrap().type_id(),
        TypeId::of::<Anonymous>()
    );
    assert!(graph.model(TypeId::of::<u32>()).is_none());
    assert!(graph.query(TypeId::of::<Target>()).is_some());
    let declarations = graph
        .query(TypeId::of::<Target>())
        .expect("entity query")
        .declarations();
    assert_eq!(declarations.len(), 1);
    assert_eq!(declarations[0].field().name(), Some("id"));
    assert_eq!(declarations[0].path().segments(), &["id"]);
    assert!(declarations[0].reasons().contains(IndexingReasons::IDENTIFIER));
    assert!(graph.query(TypeId::of::<Anonymous>()).is_none());
    assert_eq!(
        graph
            .projection_source(TypeId::of::<TargetView>())
            .unwrap()
            .target()
            .type_id(),
        TypeId::of::<Target>()
    );
    assert!(graph.projection_source(TypeId::of::<Anonymous>()).is_none());
    let location = roots[0].fields()[0].location().unwrap();
    assert_eq!(location.owner(), TypeId::of::<Anonymous>());
    assert_eq!(location.variant(), None);
    assert_eq!(location.index(), 0);
    assert!(graph.reference(location).is_none());
}

#[test]
fn test_variant_fields_have_distinct_declaration_identities() {
    let metadata = TypeMetadata::of::<Choice>();
    let variants = metadata.as_enum().unwrap().variants();
    let first = variants[0].fields()[0].location().unwrap();
    let second = variants[1].fields()[0].location().unwrap();
    assert_eq!(first.owner(), TypeId::of::<Choice>());
    assert_eq!(first.index(), second.index());
    assert_eq!(first.variant(), Some(0));
    assert_eq!(second.variant(), Some(1));
    assert_ne!(first, second);
}

#[cfg(feature = "generic")]
#[Model]
struct Generic<T> {
    value: T,
}

#[cfg(feature = "generic")]
#[test]
fn test_generic_instances_have_distinct_owners_and_definition_has_no_location() {
    let text = TypeMetadata::of::<Generic<String>>();
    let number = TypeMetadata::of::<Generic<u32>>();
    let text_field = text.fields()[0].location().unwrap();
    let number_field = number.fields()[0].location().unwrap();
    assert_eq!(text_field.owner(), TypeId::of::<Generic<String>>());
    assert_eq!(number_field.owner(), TypeId::of::<Generic<u32>>());
    assert_ne!(text_field, number_field);
    assert!(text.generic_definition().unwrap().fields()[0].location().is_none());
}
