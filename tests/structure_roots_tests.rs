// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Explicit anonymous roots and generic model definitions.

#![cfg(feature = "generic")]

use std::any::TypeId;
#[cfg(feature = "validation")]
use std::sync::Arc;

use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::Projection;
use qubit_model_derive::Value;
use qubit_model_metadata::metadata::ModelRole;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ModelGraph;
use qubit_model_metadata::resolve::ResolveErrorKind;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
#[cfg(feature = "validation")]
use qubit_model_metadata::validation::ValidationBuildInputs;
#[cfg(feature = "validation")]
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::ReflectionContext;
use qubit_reflect::descriptor::TypeKind;
#[cfg(feature = "validation")]
use qubit_validator::ValidatorRegistry;

fn linked_reflection_context() -> ReflectionContext {
    ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("linked reflection context")
}

#[Model]
struct Page<T> {
    items: Vec<T>,
}

/// Returns a graph after the resolver's locally owned root array is gone.
fn graph_from_local_roots() -> ModelGraph<'static> {
    let root = TypeMetadata::of::<Page<String>>();
    StructureResolver::for_static_roots(ModelRegistry::global(), [root])
        .resolve()
        .expect("anonymous root graph survives its input array")
}

/// Root input storage does not limit the lifetime of a graph backed by a
/// process-wide registry.
#[test]
fn test_root_graph_lifetime_follows_registry() {
    let root = TypeMetadata::of::<Page<String>>();
    let graph = graph_from_local_roots();
    assert!(graph.model(root.type_id()).is_some());
}

/// Registration identity is independent from generic model capability.
#[test]
fn test_anonymous_generic_has_definition() {
    let meta = TypeMetadata::of::<Page<String>>();
    assert!(meta.model_id().is_none());
    assert!(!meta.is_registered());
    assert!(
        meta.generic_definition()
            .expect("generic definition")
            .model_id()
            .is_none()
    );
}

/// Explicit anonymous roots participate without acquiring fabricated IDs.
#[test]
fn test_anonymous_root_is_resolved() {
    let reflection = linked_reflection_context();
    let registry = ModelRegistry::from_context(&reflection).expect("model registry");
    let root = TypeMetadata::of::<Page<String>>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs {
        models: &registry,
        roots: &roots,
    })
    .resolve()
    .expect("anonymous root resolves");
    assert!(graph.properties(root).is_some());
}

/// Execution binding accepts an anonymous concrete root without a synthetic ID.
#[cfg(feature = "validation")]
#[test]
fn test_anonymous_root_validation_binds() {
    let reflection = linked_reflection_context();
    let registry = ModelRegistry::from_context(&reflection).expect("model registry");
    let root = TypeMetadata::of::<Page<String>>();
    let roots = [root];
    let graph = Arc::new(
        StructureResolver::new(ResolveInputs {
            models: &registry,
            roots: &roots,
        })
        .resolve()
        .expect("root graph"),
    );
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: Arc::clone(&graph),
            validators: &validators,
        },
    )
    .expect("anonymous validation plan");
    assert!(plan.root().model_id().is_none());
}

#[Model(eq, hash)]
struct Owner {
    name: String,
}

#[Entity(id = "roots.Record", eq, hash)]
struct Record {
    #[identifier]
    id: Id,
    #[indexed]
    owner: Owner,
    #[indexed]
    owner_name: String,
}

/// Metadata retains direct declarations without imposing flattened filters.
#[test]
fn test_query_declarations_do_not_apply_product_filter_policy() {
    let reflection = linked_reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &[],
    })
    .resolve()
    .expect("indexed model is a valid declaration");
    let root = TypeMetadata::of::<Record>();
    let declarations = graph.query(root.type_id()).expect("query declarations").declarations();
    let names: Vec<_> = declarations
        .iter()
        .map(|value| value.field().name().expect("named field"))
        .collect();
    assert_eq!(names, ["id", "owner", "owner_name"]);
}

#[Model]
struct References {
    #[reference(entity = Record, property = "id")]
    records: Option<Vec<Id>>,
}

/// Reference compatibility unwraps supported containers while retaining shape.
#[test]
fn test_optional_sequence_reference_resolves() {
    let reflection = linked_reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let root = TypeMetadata::of::<References>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("wrapped reference");
    assert!(
        graph
            .reference(root.fields()[0].location().expect("root reference field location"))
            .is_some()
    );
    let reflected = root.fields()[0]
        .reflect_in(&reflection)
        .expect("concrete reference field");
    assert!(matches!(
        reflected.find_concrete_type().expect("outer field type").kind(),
        TypeKind::Optional(_)
    ));
}

#[Enum]
enum ReferencedValue {
    Record(#[reference(entity = Record, property = "id")] Id),
}

#[Value]
struct ClosedValue {
    link: ReferencedValue,
}

/// A Value cannot conceal an entity reference inside an Enum payload.
#[test]
fn test_value_closure_rejects_enum_payload_references() {
    let reflection = linked_reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let roots = [TypeMetadata::of::<ClosedValue>()];
    let result = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve();
    let errors = result.expect_err("Value closure must reject hidden references");
    let error = errors
        .errors()
        .iter()
        .find(|error| error.kind() == ResolveErrorKind::InvalidValueClosure)
        .expect("closure error");
    assert_eq!(error.owner_type_id(), Some(TypeMetadata::of::<ClosedValue>().type_id()));
    let location = error.declaration().expect("payload declaration");
    assert_eq!(location.variant, Some(0));
    assert_eq!(location.field, Some(0));
}

#[Entity(id = "roots.Nation")]
struct Nation {
    #[identifier]
    id: Id,
}

#[Entity(id = "roots.Zone")]
struct Zone {
    #[identifier]
    id: Id,
    #[reference(entity = Nation, property = id)]
    nation: Id,
}

#[Model]
struct BoundAddress {
    #[reference(entity = Zone, property = id)]
    zone: Id,
    #[reference(entity = Nation, property = id, path = "zone/nation")]
    nation: Id,
}

/// Binding paths traverse referenced entities even when fields only store IDs.
#[test]
fn test_binding_path_navigates_entity_behind_saved_id() {
    let reflection = linked_reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let root = TypeMetadata::of::<BoundAddress>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("binding navigation follows Entity metadata");
    assert_eq!(
        graph
            .reference(root.fields()[1].location().expect("bound nation field location"))
            .expect("bound nation reference")
            .target()
            .type_id(),
        TypeMetadata::of::<Nation>().type_id()
    );
}

#[Model]
struct InvalidDependency {
    #[validator(id = "roots.rule", depends_on(expected(property = missing)))]
    value: String,
}

/// Known local dependency paths are checked without an execution registry.
#[test]
fn test_missing_local_dependency_is_a_structure_error() {
    let reflection = linked_reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let roots = [TypeMetadata::of::<InvalidDependency>()];
    let errors = match StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    {
        Err(errors) => errors,
        Ok(_) => panic!("missing dependency must fail structural resolution"),
    };
    let error = errors
        .errors()
        .iter()
        .find(|error| error.owner_type_id() == Some(roots[0].type_id()))
        .expect("invalid dependency resolution reports an owning error");
    assert_eq!(error.kind(), ResolveErrorKind::MissingProperty);
    assert_eq!(error.path().expect("missing dependency path").to_string(), "missing");
    assert!(error.declaration().is_some());
}

#[Enum]
enum UnboundEntityPayload {
    Record(Record),
}

/// Entity payloads need a reference declaration even inside an Enum.
#[test]
fn test_enum_entity_payload_requires_reference() {
    let reflection = linked_reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let metadata = TypeMetadata::of::<UnboundEntityPayload>();
    let roots = [metadata];
    let result = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve();
    let errors = match result {
        Err(errors) => errors,
        Ok(_) => panic!("unbound Entity payload must fail"),
    };
    assert!(
        errors
            .errors()
            .iter()
            .any(|error| error.kind() == ResolveErrorKind::InvalidEntityNesting
                && error.owner_type_id() == Some(metadata.type_id()))
    );
}

#[Model]
struct SavedIdDependency {
    #[reference(entity = Record, property = id)]
    owner: Id,
    #[validator(id = "roots.rule", depends_on(expected(path = "owner", property = owner_name)))]
    value: String,
}

/// Validator navigation reads stored values and cannot load a referenced
/// Entity.
#[test]
fn test_validator_navigation_does_not_follow_entity_bindings() {
    let reflection = linked_reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let metadata = TypeMetadata::of::<SavedIdDependency>();
    let roots = [metadata];
    let result = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve();
    let errors = match result {
        Err(errors) => errors,
        Ok(_) => panic!("an ID does not expose Entity properties"),
    };
    let error = errors
        .errors()
        .iter()
        .find(|error| error.owner_type_id() == Some(metadata.type_id()))
        .expect("dependency error");
    assert_eq!(error.kind(), ResolveErrorKind::MissingProperty);
    assert_eq!(
        error.object_path().expect("dependency object path").to_string(),
        "owner"
    );
}

#[Model]
struct NestedDependency {
    child: Option<Box<DependencyChild>>,
    #[validator(id = "roots.rule", depends_on(expected(property = child.name)))]
    value: String,
}

#[Model]
struct DependencyChild {
    name: String,
}

/// Property selection unwraps optional and pointer storage, without loading
/// referenced Entities or traversing collection elements implicitly.
#[test]
fn test_dependency_property_selection_unwraps_storage() {
    let reflection = linked_reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let roots = [TypeMetadata::of::<NestedDependency>()];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("optional boxed child property is structurally readable");
    assert_eq!(graph.dependencies().len(), 1);
}

#[Model]
struct WrongReferenceType {
    #[reference(entity = Record, property = owner_name)]
    value: u32,
}

/// Cross-model type mismatches retain exact identities, roles and source
/// coordinates for consumers of structured diagnostics.
#[test]
fn test_reference_type_mismatch_retains_machine_readable_context() {
    let reflection = linked_reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let owner = TypeMetadata::of::<WrongReferenceType>();
    let roots = [owner];
    let errors = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect_err("u32 cannot store the selected String property");
    assert_eq!(errors.to_string(), "1 model resolution error(s)");
    let errors = errors.into_vec();
    let error = &errors[0];
    assert_eq!(error.kind(), ResolveErrorKind::TypeMismatch);
    assert_eq!(error.expected_type(), Some(TypeId::of::<String>()));
    assert_eq!(error.actual_type(), Some(TypeId::of::<u32>()));
    assert_eq!(error.expected_role(), Some(ModelRole::Entity));
    assert_eq!(error.actual_role(), Some(ModelRole::Entity));
    assert_eq!(error.owner_type_id(), Some(owner.type_id()));
    assert_eq!(
        error.declaration().expect("reference declaration location").field,
        Some(0)
    );
    assert_eq!(
        error.path().expect("mismatched property path").to_string(),
        "owner_name"
    );
    assert!(error.to_string().contains("TypeMismatch"));
}

#[Model]
struct MissingUniqueScope {
    #[unique(respect_to(missing))]
    value: String,
}

/// A missing uniqueness scope identifies the field that declared it.
#[test]
fn test_unique_scope_error_retains_declaration_location() {
    let reflection = linked_reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let roots = [TypeMetadata::of::<MissingUniqueScope>()];
    let errors = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect_err("missing scope");
    let error = &errors.errors()[0];
    assert_eq!(error.kind(), ResolveErrorKind::MissingProperty);
    assert_eq!(error.path().expect("missing scope path").to_string(), "missing");
    assert_eq!(
        error.declaration().expect("unique scope declaration location").field,
        Some(0)
    );
}

#[Model]
struct NationDetails {
    code: String,
}

#[Entity(id = "roots.NestedNation")]
struct NestedNation {
    #[identifier]
    id: Id,
    details: NationDetails,
}

#[Projection(source = NestedNation)]
struct NestedNationProjection {
    #[identifier]
    id: Id,
    details: NationDetails,
}

#[Model]
struct NestedNationSlot {
    nation: NestedNation,
    projection: NestedNationProjection,
}

#[Model]
struct NestedRelationRoot {
    subject: NestedNationSlot,
    #[reference(entity = NestedNation, property = id, path = "subject/nation")]
    nation_id: Id,
    #[validator(
        id = "roots.nested_dependency",
        depends_on(expected(path = "subject/nation", property = details.code))
    )]
    digest: String,
}

/// Nested model traversal resolves Entity references, Projection identifiers,
/// and validator dependencies through the same root graph.
#[test]
fn test_nested_entity_projection_and_dependency_paths_resolve() {
    let reflection = linked_reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let metadata = TypeMetadata::of::<NestedRelationRoot>();
    let roots = [metadata];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("nested relation graph resolves");

    let reference = graph
        .reference(
            metadata
                .field("nation_id")
                .expect("nation_id field metadata")
                .location()
                .expect("nation_id field location"),
        )
        .expect("nested Entity reference");
    assert_eq!(
        reference.target().type_id(),
        TypeMetadata::of::<NestedNation>().type_id()
    );
    assert_eq!(reference.property().expect("referenced Entity property").name(), "id");

    let projection = TypeMetadata::of::<NestedNationProjection>();
    assert_eq!(
        graph
            .projection_source(projection.type_id())
            .expect("nested projection source")
            .target()
            .type_id(),
        TypeMetadata::of::<NestedNation>().type_id()
    );

    let dependency = &graph.dependencies()[0];
    assert_eq!(dependency.declaration().object_path().to_string(), "subject/nation");
    assert_eq!(dependency.declaration().property().to_string(), "details.code");
}

#[Model]
struct InvalidNestedRelationRoot {
    subject: NestedNationSlot,
    #[reference(entity = NestedNation, property = id, path = "subject/missing")]
    nation_id: Id,
    #[validator(
        id = "roots.invalid_nested_dependency",
        depends_on(expected(path = "subject/nation", property = details.missing))
    )]
    digest: String,
}

/// Broken nested reference and dependency paths retain both path components
/// and their declaring field locations.
#[test]
fn test_nested_relation_errors_retain_exact_object_and_property_paths() {
    let reflection = linked_reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let metadata = TypeMetadata::of::<InvalidNestedRelationRoot>();
    let roots = [metadata];
    let errors = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect_err("missing nested relation paths are rejected");

    let reference_error = errors
        .errors()
        .iter()
        .find(|error| error.declaration().is_some_and(|location| location.field == Some(1)))
        .expect("reference path error");
    assert_eq!(reference_error.kind(), ResolveErrorKind::MissingProperty);
    assert_eq!(
        reference_error
            .object_path()
            .expect("invalid reference object path")
            .to_string(),
        "subject/missing"
    );
    assert_eq!(
        reference_error
            .path()
            .expect("invalid reference field path")
            .to_string(),
        "nation_id"
    );

    let dependency_error = errors
        .errors()
        .iter()
        .find(|error| error.declaration().is_some_and(|location| location.field == Some(2)))
        .expect("dependency path error");
    assert_eq!(dependency_error.kind(), ResolveErrorKind::MissingProperty);
    assert_eq!(
        dependency_error
            .object_path()
            .expect("invalid dependency object path")
            .to_string(),
        "subject/nation"
    );
    assert_eq!(
        dependency_error
            .path()
            .expect("invalid dependency property path")
            .to_string(),
        "details.missing"
    );
}
