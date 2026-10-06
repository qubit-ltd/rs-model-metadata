// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Explicit anonymous roots and generic model definitions.

#![cfg(feature = "generic")]

#[cfg(feature = "validation")]
use std::sync::Arc;
use std::sync::OnceLock;

use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::Value;
use qubit_model_metadata::__private::ModelMetadataProvider;
use qubit_model_metadata::__private::model_metadata_key;
use qubit_model_metadata::__private::v7;
use qubit_model_metadata::metadata::DependencyBindingMetadata;
use qubit_model_metadata::metadata::FieldAttributeMetadata;
use qubit_model_metadata::metadata::ModelId;
use qubit_model_metadata::metadata::ModelRole;
use qubit_model_metadata::metadata::OnNone;
use qubit_model_metadata::metadata::PropertyPath;
use qubit_model_metadata::metadata::SerdeFieldMetadata;
use qubit_model_metadata::metadata::TargetMode;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::metadata::ValidatorMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ModelGraph;
use qubit_model_metadata::resolve::ResolveErrorKind;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
#[cfg(feature = "validation")]
use qubit_model_metadata::validation::ValidationBuildInputs;
#[cfg(feature = "validation")]
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::Reflect;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::capability::CapabilityDescriptor;
use qubit_reflect::identity::FragmentIdentity;
use qubit_reflect::registry::RegistrySnapshotBuilder;
#[cfg(feature = "validation")]
use qubit_validator::ValidatorRegistry;

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

#[derive(Reflect)]
struct SnapshotGood {
    value: String,
}

#[derive(Reflect)]
struct SnapshotBad {
    value: String,
}

/// Supplies metadata for the valid snapshot member.
fn snapshot_good_metadata() -> &'static TypeMetadata {
    static METADATA: OnceLock<TypeMetadata> = OnceLock::new();
    METADATA.get_or_init(|| {
        let descriptor = TypeDescriptor::of::<SnapshotGood>();
        let reflected = descriptor.field_at(0).expect("good value field");
        let fields = v7::leak_slice(vec![v7::field_metadata(
            reflected.declaring_type().type_id(),
            reflected,
            &[],
            &[],
            &[],
            &SerdeFieldMetadata::DEFAULT,
        )]);
        let properties = v7::leak_slice(vec![v7::property_metadata(
            "value",
            fields[0].type_ref(),
            Some(&fields[0]),
            None,
            None,
        )]);
        v7::GeneratedTypeMetadataBuilder::new(
            descriptor,
            Some(ModelId::new("roots.SnapshotGood")),
            fields,
            v7::leak(v7::model_role()),
        )
        .properties(properties)
        .finish::<SnapshotGood>()
    })
}

/// Supplies metadata for the invalid snapshot member.
fn snapshot_bad_metadata() -> &'static TypeMetadata {
    static METADATA: OnceLock<TypeMetadata> = OnceLock::new();
    METADATA.get_or_init(|| {
        let descriptor = TypeDescriptor::of::<SnapshotBad>();
        let reflected = descriptor.field_at(0).expect("bad value field");
        let missing = PropertyPath::new(&["missing"]);
        let dependencies = v7::leak_slice(vec![missing]);
        let bindings = v7::leak_slice(vec![DependencyBindingMetadata::new("expected", missing)]);
        let validators = v7::leak_slice(vec![ValidatorMetadata::new_bound(
            "roots.rule",
            &[],
            dependencies,
            bindings,
            TargetMode::Value,
            OnNone::Skip,
        )]);
        let attributes = v7::leak_slice(vec![FieldAttributeMetadata::Validator(&validators[0])]);
        let fields = v7::leak_slice(vec![v7::field_metadata(
            reflected.declaring_type().type_id(),
            reflected,
            attributes,
            &[],
            validators,
            &SerdeFieldMetadata::DEFAULT,
        )]);
        let properties = v7::leak_slice(vec![v7::property_metadata(
            "value",
            fields[0].type_ref(),
            Some(&fields[0]),
            None,
            None,
        )]);
        v7::GeneratedTypeMetadataBuilder::new(
            descriptor,
            Some(ModelId::new("roots.SnapshotBad")),
            fields,
            v7::leak(v7::model_role()),
        )
        .properties(properties)
        .finish::<SnapshotBad>()
    })
}

/// A frozen reflection snapshot isolates valid roots from unrelated defects.
#[test]
fn test_for_roots_ignores_unrelated_invalid_snapshot_member() {
    let mut builder = RegistrySnapshotBuilder::new();
    for (metadata, provider, member) in [
        (
            snapshot_good_metadata(),
            snapshot_good_metadata as ModelMetadataProvider,
            "good",
        ),
        (
            snapshot_bad_metadata(),
            snapshot_bad_metadata as ModelMetadataProvider,
            "bad",
        ),
    ] {
        builder.add_type_with_capabilities(
            metadata.descriptor(),
            vec![CapabilityDescriptor::with_adapter(
                model_metadata_key(),
                provider,
            )],
            FragmentIdentity::new("roots-test", member, 1, 1, "type", 0),
            FragmentIdentity::new("roots-test", member, 1, 2, "capability", 0),
        );
    }
    let reflection = builder.build().expect("isolated reflection snapshot");
    let models =
        ModelRegistry::from_reflect_registry(&reflection).expect("snapshot model projection");
    let root = snapshot_good_metadata();
    let roots = [root];
    let inputs = ResolveInputs {
        models: &models,
        roots: &roots,
    };

    let full_errors = StructureResolver::new(inputs)
        .resolve()
        .expect_err("complete audit visits the unrelated bad model");
    assert!(full_errors.errors().iter().any(|error| {
        error.kind() == ResolveErrorKind::MissingProperty
            && error.owner_type_id() == Some(snapshot_bad_metadata().type_id())
    }));

    let graph = StructureResolver::for_roots(inputs)
        .resolve()
        .expect("root-scoped resolution ignores the bad model");
    assert!(graph.model(root.type_id()).is_some());
    assert!(graph.model(snapshot_bad_metadata().type_id()).is_none());
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
    let registry = ModelRegistry::global();
    let root = TypeMetadata::of::<Page<String>>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs {
        models: registry,
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
    let registry = ModelRegistry::global();
    let root = TypeMetadata::of::<Page<String>>();
    let roots = [root];
    let graph = Arc::new(
        StructureResolver::new(ResolveInputs {
            models: registry,
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
    let models = ModelRegistry::global();
    let graph = StructureResolver::new(ResolveInputs { models, roots: &[] })
        .resolve()
        .expect("indexed model is a valid declaration");
    let root = TypeMetadata::of::<Record>();
    let declarations = graph
        .query(root.type_id())
        .expect("query declarations")
        .declarations();
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
    let models = ModelRegistry::global();
    let root = TypeMetadata::of::<References>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs {
        models,
        roots: &roots,
    })
    .resolve()
    .expect("wrapped reference");
    assert!(
        graph
            .reference(root.fields()[0].location().unwrap())
            .is_some()
    );
    assert!(
        root.fields()[0]
            .descriptor()
            .expect("outer shape")
            .as_optional()
            .is_some()
    );
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
    let models = ModelRegistry::global();
    let roots = [TypeMetadata::of::<ClosedValue>()];
    let result = StructureResolver::new(ResolveInputs {
        models,
        roots: &roots,
    })
    .resolve();
    let errors = result.expect_err("Value closure must reject hidden references");
    let error = errors
        .errors()
        .iter()
        .find(|error| error.kind() == ResolveErrorKind::InvalidValueClosure)
        .expect("closure error");
    assert_eq!(
        error.owner_type_id(),
        Some(TypeMetadata::of::<ClosedValue>().type_id())
    );
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
    let models = ModelRegistry::global();
    let root = TypeMetadata::of::<BoundAddress>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs {
        models,
        roots: &roots,
    })
    .resolve()
    .expect("binding navigation follows Entity metadata");
    assert_eq!(
        graph
            .reference(root.fields()[1].location().unwrap())
            .unwrap()
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
    let models = ModelRegistry::global();
    let roots = [TypeMetadata::of::<InvalidDependency>()];
    let errors = match StructureResolver::new(ResolveInputs {
        models,
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
        .unwrap();
    assert_eq!(error.kind(), ResolveErrorKind::MissingProperty);
    assert_eq!(error.path().unwrap().to_string(), "missing");
    assert!(error.declaration().is_some());
}

#[Enum]
enum UnboundEntityPayload {
    Record(Record),
}

/// Entity payloads need a reference declaration even inside an Enum.
#[test]
fn test_enum_entity_payload_requires_reference() {
    let metadata = TypeMetadata::of::<UnboundEntityPayload>();
    let roots = [metadata];
    let result = StructureResolver::new(ResolveInputs {
        models: ModelRegistry::global(),
        roots: &roots,
    })
    .resolve();
    let errors = match result {
        Err(errors) => errors,
        Ok(_) => panic!("unbound Entity payload must fail"),
    };
    assert!(errors.errors().iter().any(|error| error.kind()
        == ResolveErrorKind::InvalidEntityNesting
        && error.owner_type_id() == Some(metadata.type_id())));
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
    let metadata = TypeMetadata::of::<SavedIdDependency>();
    let roots = [metadata];
    let result = StructureResolver::new(ResolveInputs {
        models: ModelRegistry::global(),
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
    assert_eq!(error.object_path().unwrap().to_string(), "owner");
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
    let roots = [TypeMetadata::of::<NestedDependency>()];
    let graph = StructureResolver::new(ResolveInputs {
        models: ModelRegistry::global(),
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
    let owner = TypeMetadata::of::<WrongReferenceType>();
    let roots = [owner];
    let errors = StructureResolver::new(ResolveInputs {
        models: ModelRegistry::global(),
        roots: &roots,
    })
    .resolve()
    .expect_err("u32 cannot store the selected String property");
    assert_eq!(errors.to_string(), "1 model resolution error(s)");
    let errors = errors.into_vec();
    let error = &errors[0];
    assert_eq!(error.kind(), ResolveErrorKind::TypeMismatch);
    assert_eq!(
        error.expected_type(),
        Some(std::any::TypeId::of::<String>())
    );
    assert_eq!(error.actual_type(), Some(std::any::TypeId::of::<u32>()));
    assert_eq!(error.expected_role(), Some(ModelRole::Entity));
    assert_eq!(error.actual_role(), Some(ModelRole::Entity));
    assert_eq!(error.owner_type_id(), Some(owner.type_id()));
    assert_eq!(error.declaration().unwrap().field, Some(0));
    assert_eq!(error.path().unwrap().to_string(), "owner_name");
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
    let roots = [TypeMetadata::of::<MissingUniqueScope>()];
    let errors = StructureResolver::new(ResolveInputs {
        models: ModelRegistry::global(),
        roots: &roots,
    })
    .resolve()
    .expect_err("missing scope");
    let error = &errors.errors()[0];
    assert_eq!(error.kind(), ResolveErrorKind::MissingProperty);
    assert_eq!(error.path().unwrap().to_string(), "missing");
    assert_eq!(error.declaration().unwrap().field, Some(0));
}
