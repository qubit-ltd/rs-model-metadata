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
use qubit_model_derive::ModelImpl;
use qubit_model_derive::Projection;
use qubit_model_metadata::__private::v8;
use qubit_model_metadata::metadata::IndexingReasons;
use qubit_model_metadata::metadata::PropertyValue;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ContextRequirement;
use qubit_model_metadata::resolve::ResolveErrorKind;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_reflect::ReflectionContext;
use qubit_reflect::value::DynamicRef;
use qubit_reflect::value::Local;

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

#[Model(id = "identity.RustTypeOverlayRoot")]
struct RustTypeOverlayRoot {
    #[reference(entity = Target)]
    target: Target,
}

#[Model(id = "identity.IdRoot")]
struct ModelIdRoot {
    #[reference(entity_id = "identity.Target")]
    target: Target,
}

#[test]
fn test_reference_targets_use_the_explicit_overlay_for_both_target_kinds() {
    let models = ModelRegistry::from_context(ReflectionContext::global()).expect("valid registry");
    let registered = TypeMetadata::of::<Target>();
    let overlay = v8::leak(
        v8::GeneratedTypeMetadataBuilder::new::<Target>(
            registered.model_id(),
            registered.fields(),
            registered.role_metadata(),
        )
        .finish::<Target>(),
    );
    assert!(!std::ptr::eq(registered, overlay));
    let root = TypeMetadata::of::<RustTypeOverlayRoot>();
    let model_id_root = TypeMetadata::of::<ModelIdRoot>();
    let roots = [overlay, root, model_id_root];
    let graph = StructureResolver::for_roots(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("both reference target forms resolve");
    assert!(std::ptr::eq(
        graph.model(overlay.type_id()).expect("graph target"),
        overlay
    ));
    for owner in [root, model_id_root] {
        let field = owner.field("target").expect("reference field");
        let resolved = graph
            .reference(field.location().expect("reference location"))
            .expect("resolved target");
        assert!(std::ptr::eq(resolved.target(), overlay));
        assert!(std::ptr::eq(
            graph.model(resolved.target().type_id()).unwrap(),
            resolved.target()
        ));
    }
}

#[test]
fn test_reference_lookup_preserves_copied_field_identity() {
    let models = ModelRegistry::from_context(ReflectionContext::global()).expect("valid registry");
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &[],
    })
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
    assert!(std::ptr::eq(graph.registry(), &models));
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
    let models = ModelRegistry::from_context(ReflectionContext::global()).expect("valid registry");
    let parent = TypeMetadata::of::<ParentBoundReference>();
    let whole = TypeMetadata::of::<WholeEntityReference>();
    let roots = [parent, whole];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
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
    let errors = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
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

#[Entity(id = "identity.ProjectionSource")]
struct ProjectionSource {
    #[identifier]
    id: Id,
}

#[Projection(source = ProjectionSource)]
struct ProducedView {
    #[identifier]
    id: Id,
}

#[ModelImpl]
impl ProjectionSource {
    pub fn view(&self) -> ProducedView {
        ProducedView { id: self.id }
    }
}

#[test]
fn test_anonymous_roots_and_role_queries_use_concrete_type_identity() {
    let models = ModelRegistry::from_context(ReflectionContext::global()).expect("valid registry");
    let roots = [TypeMetadata::of::<Anonymous>(), TypeMetadata::of::<TargetView>()];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
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

#[test]
fn test_resolved_projection_producer_executes_and_preserves_identifier() {
    let models = ModelRegistry::from_context(ReflectionContext::global()).expect("valid registry");
    let roots = [TypeMetadata::of::<ProjectionSource>()];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("valid projection producer graph");

    let [producer] = graph.projection_producers() else {
        panic!("the registered getter should produce one projection edge");
    };
    assert_eq!(producer.source().type_id(), TypeId::of::<ProjectionSource>());
    assert_eq!(producer.projection().type_id(), TypeId::of::<ProducedView>());
    assert_eq!(producer.property().name(), "view");
    assert!(producer.projector().is_some());

    let source = ProjectionSource { id: Id::new(41) };
    let PropertyValue::Owned(projected) = producer
        .project(DynamicRef::<Local>::new(&source))
        .expect("getter should return a view with the source identifier")
    else {
        panic!("fixture getter returns an owned view");
    };
    assert_eq!(projected.downcast::<ProducedView>().unwrap().id, source.id);
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
