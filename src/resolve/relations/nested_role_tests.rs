use std::any::TypeId;
use std::collections::HashSet;

use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::Projection;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::capability::CapabilityRegistration;
use qubit_reflect::capability::CapabilitySpec;
use serde::Deserialize;
use serde::Serialize;

use super::super::internal::ResolutionContext;
use super::forbidden_entity_nested_role;
use super::forbidden_entity_nested_role_with_visit;
use crate::__private::v8;
use crate::metadata::FieldMetadata;
use crate::metadata::ModelMetadataError;
use crate::metadata::ModelRole;
use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;
use crate::structure::children;

#[Model]
struct SharedLeaf {
    value: u8,
}

#[Model]
struct SharedLevel1 {
    left: SharedLeaf,
    right: SharedLeaf,
}

#[Model]
struct SharedLevel2 {
    left: SharedLevel1,
    right: SharedLevel1,
}

#[Model]
struct SharedLevel4 {
    left: SharedLevel2,
    right: SharedLevel2,
}

#[Model]
struct SharedLevel8 {
    left: SharedLevel4,
    right: SharedLevel4,
}

#[Model]
struct SharedLevel12 {
    left: SharedLevel8,
    right: SharedLevel8,
}

#[Entity(id = "nested.role.Entity", eq, hash)]
struct ForbiddenEntity {
    #[identifier]
    id: Id,
}

#[Projection(source = ForbiddenEntity)]
struct ForbiddenProjection {
    #[identifier]
    id: Id,
}

#[Model]
struct CycleThenEntity {
    next: Option<Box<CycleThenEntity>>,
    forbidden: ForbiddenEntity,
}

#[Model]
struct CycleThenProjection {
    next: Option<Box<CycleThenProjection>>,
    forbidden: ForbiddenProjection,
}

#[Model]
struct EntityThenProjection {
    first: ForbiddenEntity,
    second: ForbiddenProjection,
}

#[Model]
struct ProjectionThenEntity {
    first: ForbiddenProjection,
    second: ForbiddenEntity,
}

#[Model]
struct OpaqueAndReferenceBoundary {
    #[opaque]
    hidden: ForbiddenEntity,
    #[reference(entity = ForbiddenEntity)]
    referenced: ForbiddenEntity,
}

#[Enum]
enum EntityPayload {
    Nested(Option<Box<Vec<(ForbiddenEntity, ForbiddenEntity)>>>),
}

#[Model]
struct ContainerPayload {
    nested: Option<Box<Vec<(ForbiddenEntity, ForbiddenEntity)>>>,
}

#[Model]
struct EnumPayload {
    nested: EntityPayload,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[Reflect]
struct RawEntityPayload {
    nested: Option<Box<Vec<(ForbiddenEntity, ForbiddenEntity)>>>,
}

struct InvalidMetadataCapability;

impl CapabilitySpec for InvalidMetadataCapability {
    type Adapter = u32;

    const ID: &'static str = "qubit.model.metadata.v1";
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[Reflect]
struct LookupFailure;

#[Model]
struct ErrorThenEntity {
    failure: LookupFailure,
    forbidden: ForbiddenEntity,
}

#[Model]
struct EntityThenError {
    forbidden: ForbiddenEntity,
    failure: LookupFailure,
}

#[Model]
struct CycleThenError {
    next: Option<Box<CycleThenError>>,
    failure: LookupFailure,
}

type LegacyResult = Result<(Option<ModelRole>, Vec<TypeId>), ModelMetadataError>;

fn reflection_context() -> ReflectionContext {
    ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("linked reflection context")
}

fn lookup_failure_reflection() -> ReflectionContext {
    ReflectionContext::builder()
        .include_linked()
        .register(CapabilityRegistration::for_type::<
            LookupFailure,
            InvalidMetadataCapability,
        >(7_u32))
        .build()
        .expect("wrongly typed metadata capability is a valid reflection fixture")
}

fn previous_algorithm<'ctx>(
    descriptor: TypeDescriptor<'ctx>,
    context: &ResolutionContext<'ctx>,
) -> LegacyResult {
    let mut ancestors = HashSet::new();
    let mut pending = vec![(descriptor, false)];
    let mut visits = Vec::new();
    while let Some((current, exiting)) = pending.pop() {
        if exiting {
            ancestors.remove(&current.type_id());
            continue;
        }
        if !ancestors.insert(current.type_id()) {
            continue;
        }
        visits.push(current.type_id());
        pending.push((current, true));
        let first_child = pending.len();
        if let Some(metadata) = super::metadata_for_descriptor(current, context)? {
            if matches!(metadata.role(), ModelRole::Entity | ModelRole::Projection) {
                return Ok((Some(metadata.role()), visits));
            }
            let variants = metadata
                .as_enum()
                .into_iter()
                .flat_map(|enumeration| enumeration.variants())
                .flat_map(|variant| variant.fields());
            for field in metadata.fields().iter().chain(variants) {
                if !field.is_opaque()
                    && field.reference().is_none()
                    && let Some(child) = metadata
                        .field_descriptor_in(field, context.reflection())
                        .and_then(|field| field.find_concrete_type())
                {
                    pending.push((child, false));
                }
            }
        } else {
            children(current, |edge| pending.push((edge.target, false)));
        }
        pending[first_child..].reverse();
    }
    Ok((None, visits))
}

fn assert_matches_previous<'ctx>(
    descriptor: TypeDescriptor<'ctx>,
    context: &ResolutionContext<'ctx>,
) -> Result<Option<ModelRole>, String> {
    let current = forbidden_entity_nested_role(descriptor, context);
    let previous = previous_algorithm(descriptor, context).map(|(role, _)| role);
    assert_eq!(
        format!("{current:?}"),
        format!("{previous:?}"),
        "current and previous algorithms must preserve the same role or error cause",
    );
    current.map_err(|error| format!("{error:?}"))
}

#[test]
fn test_nested_role_shared_graph_expands_each_type_once() {
    let root = TypeMetadata::of::<SharedLevel1>();
    let reflection = reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let resolution = ResolutionContext::new(&models, &[root]);
    let mut visits = Vec::<TypeId>::new();

    let result = forbidden_entity_nested_role_with_visit(
        TypeDescriptor::of::<SharedLevel1>(),
        &resolution,
        |type_id| visits.push(type_id),
    )
    .expect("metadata lookup succeeds");

    assert_eq!(result, None);
    let unique = visits.iter().copied().collect::<HashSet<_>>();
    assert_eq!(visits.len(), unique.len(), "each descriptor expands once");
    let (_, previous) = previous_algorithm(TypeDescriptor::of::<SharedLevel1>(), &resolution)
        .expect("reference traversal has valid metadata");
    assert!(
        previous.len() > visits.len(),
        "the former traversal repeats shared nodes"
    );
}

#[test]
fn test_nested_role_shared_graph_visit_count_tracks_unique_depth() {
    let reflection = reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    for (descriptor, root, expected) in [
        (
            TypeDescriptor::of::<SharedLevel1>(),
            TypeMetadata::of::<SharedLevel1>(),
            3,
        ),
        (
            TypeDescriptor::of::<SharedLevel4>(),
            TypeMetadata::of::<SharedLevel4>(),
            5,
        ),
        (
            TypeDescriptor::of::<SharedLevel8>(),
            TypeMetadata::of::<SharedLevel8>(),
            6,
        ),
        (
            TypeDescriptor::of::<SharedLevel12>(),
            TypeMetadata::of::<SharedLevel12>(),
            7,
        ),
    ] {
        let resolution = ResolutionContext::new(&models, &[root]);
        let mut visits = Vec::new();
        assert_eq!(
            forbidden_entity_nested_role_with_visit(descriptor, &resolution, |id| visits.push(id))
                .expect("metadata lookup succeeds"),
            None,
        );
        assert_eq!(visits.len(), expected);
    }

    let root = TypeMetadata::of::<SharedLevel1>();
    let resolution = ResolutionContext::new(&models, &[root]);
    for _ in 0..2 {
        let mut visits = Vec::new();
        assert_eq!(
            forbidden_entity_nested_role_with_visit(
                TypeDescriptor::of::<SharedLevel1>(),
                &resolution,
                |id| visits.push(id),
            )
            .expect("metadata lookup succeeds"),
            None,
        );
        assert_eq!(visits.len(), 3, "each query owns a fresh seen set");
    }
}

#[test]
fn test_nested_role_cycle_does_not_hide_a_later_entity_or_projection() {
    let reflection = reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    for (descriptor, root, role) in [
        (
            TypeDescriptor::of::<CycleThenEntity>(),
            TypeMetadata::of::<CycleThenEntity>(),
            ModelRole::Entity,
        ),
        (
            TypeDescriptor::of::<CycleThenProjection>(),
            TypeMetadata::of::<CycleThenProjection>(),
            ModelRole::Projection,
        ),
    ] {
        let resolution = ResolutionContext::new(&models, &[root]);
        assert_eq!(
            assert_matches_previous(descriptor, &resolution),
            Ok(Some(role))
        );
    }
}

#[test]
fn test_nested_role_keeps_declaration_order_between_entity_and_projection() {
    let reflection = reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    for (descriptor, root, expected) in [
        (
            TypeDescriptor::of::<EntityThenProjection>(),
            TypeMetadata::of::<EntityThenProjection>(),
            ModelRole::Entity,
        ),
        (
            TypeDescriptor::of::<ProjectionThenEntity>(),
            TypeMetadata::of::<ProjectionThenEntity>(),
            ModelRole::Projection,
        ),
    ] {
        let resolution = ResolutionContext::new(&models, &[root]);
        assert_eq!(
            assert_matches_previous(descriptor, &resolution),
            Ok(Some(expected))
        );
    }
}

#[test]
fn test_nested_role_preserves_error_and_role_order_after_cycles() {
    let reflection = lookup_failure_reflection();
    let models = ModelRegistry::from_static_metadata(&reflection, &[])
        .expect("metadata-only registry defers capability lookup");

    for (descriptor, root) in [
        (
            TypeDescriptor::of::<ErrorThenEntity>(),
            TypeMetadata::of::<ErrorThenEntity>(),
        ),
        (
            TypeDescriptor::of::<CycleThenError>(),
            TypeMetadata::of::<CycleThenError>(),
        ),
    ] {
        let roots = [root, TypeMetadata::of::<ForbiddenEntity>()];
        let resolution = ResolutionContext::new(&models, &roots);
        let error = assert_matches_previous(descriptor, &resolution)
            .expect_err("a capability failure before a later role must propagate");
        assert!(error.contains("Capability"), "structured cause: {error}");
        assert!(matches!(
            forbidden_entity_nested_role(descriptor, &resolution),
            Err(ModelMetadataError::Capability { type_id, .. })
                if type_id == TypeId::of::<LookupFailure>()
        ));
    }

    let root = TypeMetadata::of::<EntityThenError>();
    let roots = [root, TypeMetadata::of::<ForbiddenEntity>()];
    let resolution = ResolutionContext::new(&models, &roots);
    assert_eq!(
        assert_matches_previous(TypeDescriptor::of::<EntityThenError>(), &resolution),
        Ok(Some(ModelRole::Entity)),
        "the first role returns before a later metadata error is queried",
    );
}

#[test]
fn test_nested_role_respects_opaque_and_reference_boundaries() {
    let root = TypeMetadata::of::<OpaqueAndReferenceBoundary>();
    let reflection = reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let resolution = ResolutionContext::new(&models, &[root]);
    assert_eq!(
        assert_matches_previous(
            TypeDescriptor::of::<OpaqueAndReferenceBoundary>(),
            &resolution,
        ),
        Ok(None),
    );
}

#[test]
fn test_nested_role_finds_entities_through_enum_raw_and_repeated_containers() {
    let reflection = reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    for (descriptor, root) in [
        (
            TypeDescriptor::of::<ContainerPayload>(),
            TypeMetadata::of::<ContainerPayload>(),
        ),
        (
            TypeDescriptor::of::<EnumPayload>(),
            TypeMetadata::of::<EnumPayload>(),
        ),
        (
            TypeDescriptor::of::<RawEntityPayload>(),
            TypeMetadata::of::<ContainerPayload>(),
        ),
    ] {
        let resolution = ResolutionContext::new(&models, &[root]);
        assert_eq!(
            assert_matches_previous(descriptor, &resolution),
            Ok(Some(ModelRole::Entity)),
        );
    }
}

#[test]
fn test_nested_role_context_and_explicit_root_overlay_are_isolated() {
    let reflection = reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let root = TypeMetadata::of::<ContainerPayload>();
    let entity = TypeMetadata::of::<ForbiddenEntity>();
    let canonical = ResolutionContext::new(&models, &[root]);
    assert_eq!(
        assert_matches_previous(TypeDescriptor::of::<ContainerPayload>(), &canonical),
        Ok(Some(ModelRole::Entity)),
    );

    let descriptor = TypeDescriptor::of::<ForbiddenEntity>();
    let fields = descriptor
        .fields()
        .iter()
        .map(|field| FieldMetadata::from_reflect(descriptor.type_id(), field))
        .collect();
    let overlay = v8::leak(
        v8::GeneratedTypeMetadataBuilder::new::<ForbiddenEntity>(
            None,
            v8::leak_slice(fields),
            v8::leak(v8::model_role()),
        )
        .finish::<ForbiddenEntity>(),
    );
    assert_eq!(overlay.type_id(), entity.type_id());
    let overlay_roots = [root, overlay];
    let overlaid = ResolutionContext::new(&models, &overlay_roots);
    assert_eq!(
        assert_matches_previous(TypeDescriptor::of::<ContainerPayload>(), &overlaid),
        Ok(None),
        "the current explicit overlay replaces the registry role for this query",
    );
    assert_eq!(
        assert_matches_previous(TypeDescriptor::of::<ContainerPayload>(), &canonical),
        Ok(Some(ModelRole::Entity)),
        "a second context cannot retain the other context's overlay",
    );
}
