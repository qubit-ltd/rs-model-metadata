// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::any::TypeId;
use std::collections::HashSet;

use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::Projection;
use qubit_reflect::__private::codegen_v4::access::FieldAccess as ReflectFieldAccess;
use qubit_reflect::__private::codegen_v4::access::FieldMetadata as ReflectFieldMetadata;
use qubit_reflect::__private::codegen_v4::access::TypeKindMetadata;
use qubit_reflect::__private::codegen_v4::access::TypedFieldAdapter;
use qubit_reflect::__private::codegen_v4::access::Visibility;
use qubit_reflect::__private::codegen_v4::descriptor::Reflect as ReflectTrait;
use qubit_reflect::__private::codegen_v4::descriptor::TypeRegistration;
use qubit_reflect::__private::codegen_v4::expression::ExpressionData;
use qubit_reflect::__private::codegen_v4::expression::GenericExpressionArgument;
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
    Nested(Option<Box<(Vec<ForbiddenEntity>, ForbiddenEntity)>>),
}

#[Model]
struct ContainerPayload {
    nested: Option<Box<(Vec<ForbiddenEntity>, ForbiddenEntity)>>,
}

#[Model]
struct EnumPayload {
    nested: EntityPayload,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[Reflect]
struct RawEntityPayload {
    nested: Option<Box<(Vec<ForbiddenEntity>, ForbiddenEntity)>>,
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

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[Reflect]
struct AbiFailureTarget;

fn wrong_abi_metadata_provider() -> &'static TypeMetadata {
    TypeMetadata::of::<SharedLeaf>()
}

#[Model]
struct AbiThenEntity {
    failure: AbiFailureTarget,
    forbidden: ForbiddenEntity,
}

#[Model]
struct EntityThenAbi {
    forbidden: ForbiddenEntity,
    failure: AbiFailureTarget,
}

// Manual registrations build a two-TypeId cycle without recursive derive
// bounds.
struct MutualCycleRoot {
    next: Box<MutualCycleNode>,
}

struct MutualCycleNode {
    back: MutualCycleRoot,
    failure: AbiFailureTarget,
    role: ForbiddenProjection,
}

const MUTUAL_ROOT_NAME: &str = concat!(module_path!(), "::MutualCycleRoot");
const MUTUAL_NODE_NAME: &str = concat!(module_path!(), "::MutualCycleNode");
const ABI_FAILURE_NAME: &str = concat!(module_path!(), "::AbiFailureTarget");
const FORBIDDEN_PROJECTION_NAME: &str = concat!(module_path!(), "::ForbiddenProjection");

#[allow(clippy::borrowed_box)] // The reflected field adapter must retain the Box field type.
fn mutual_root_next(value: &MutualCycleRoot) -> &Box<MutualCycleNode> {
    &value.next
}

fn mutual_root_next_mut(value: &mut MutualCycleRoot) -> &mut Box<MutualCycleNode> {
    &mut value.next
}

fn mutual_node_back(value: &MutualCycleNode) -> &MutualCycleRoot {
    &value.back
}

fn mutual_node_back_mut(value: &mut MutualCycleNode) -> &mut MutualCycleRoot {
    &mut value.back
}

fn mutual_node_failure(value: &MutualCycleNode) -> &AbiFailureTarget {
    &value.failure
}

fn mutual_node_failure_mut(value: &mut MutualCycleNode) -> &mut AbiFailureTarget {
    &mut value.failure
}

fn mutual_node_role(value: &MutualCycleNode) -> &ForbiddenProjection {
    &value.role
}

fn mutual_node_role_mut(value: &mut MutualCycleNode) -> &mut ForbiddenProjection {
    &mut value.role
}

impl ReflectTrait for MutualCycleRoot {
    fn __reflect_registration() -> &'static TypeRegistration<Self> {
        static ROOT_EXPRESSION: ExpressionData = ExpressionData::Named {
            path: "Box",
            definition: None,
            arguments: &BOX_NODE_ARGUMENTS,
        };
        static ROOT_ADAPTER: TypedFieldAdapter<MutualCycleRoot, Box<MutualCycleNode>> =
            TypedFieldAdapter::new(mutual_root_next, mutual_root_next_mut);
        static ROOT_FIELDS: [ReflectFieldMetadata; 1] =
            [ReflectFieldMetadata::typed::<MutualCycleRoot, Box<MutualCycleNode>>(
                0,
                "next",
                Visibility::Public,
                &ROOT_EXPRESSION,
                &ROOT_ADAPTER,
                ReflectFieldAccess::new(true, true, true, true),
            )];
        static ROOT_SHAPE: TypeKindMetadata = TypeKindMetadata::Struct(&ROOT_FIELDS);
        TypeRegistration::<Self>::get_or_init_with(|| TypeRegistration::<Self>::with_shape(&ROOT_SHAPE, None, &[]))
    }
}

impl ReflectTrait for MutualCycleNode {
    fn __reflect_registration() -> &'static TypeRegistration<Self> {
        static BACK_EXPRESSION: ExpressionData = ExpressionData::Named {
            path: MUTUAL_ROOT_NAME,
            definition: None,
            arguments: &[],
        };
        static FAILURE_EXPRESSION: ExpressionData = ExpressionData::Named {
            path: ABI_FAILURE_NAME,
            definition: None,
            arguments: &[],
        };
        static ROLE_EXPRESSION: ExpressionData = ExpressionData::Named {
            path: FORBIDDEN_PROJECTION_NAME,
            definition: None,
            arguments: &[],
        };
        static BACK_ADAPTER: TypedFieldAdapter<MutualCycleNode, MutualCycleRoot> =
            TypedFieldAdapter::new(mutual_node_back, mutual_node_back_mut);
        static FAILURE_ADAPTER: TypedFieldAdapter<MutualCycleNode, AbiFailureTarget> =
            TypedFieldAdapter::new(mutual_node_failure, mutual_node_failure_mut);
        static ROLE_ADAPTER: TypedFieldAdapter<MutualCycleNode, ForbiddenProjection> =
            TypedFieldAdapter::new(mutual_node_role, mutual_node_role_mut);
        static FIELDS: [ReflectFieldMetadata; 3] = [
            ReflectFieldMetadata::typed::<MutualCycleNode, MutualCycleRoot>(
                0,
                "back",
                Visibility::Public,
                &BACK_EXPRESSION,
                &BACK_ADAPTER,
                ReflectFieldAccess::new(true, true, true, true),
            ),
            ReflectFieldMetadata::typed::<MutualCycleNode, AbiFailureTarget>(
                1,
                "failure",
                Visibility::Public,
                &FAILURE_EXPRESSION,
                &FAILURE_ADAPTER,
                ReflectFieldAccess::new(true, true, true, true),
            ),
            ReflectFieldMetadata::typed::<MutualCycleNode, ForbiddenProjection>(
                2,
                "role",
                Visibility::Public,
                &ROLE_EXPRESSION,
                &ROLE_ADAPTER,
                ReflectFieldAccess::new(true, true, true, true),
            ),
        ];
        static SHAPE: TypeKindMetadata = TypeKindMetadata::Struct(&FIELDS);
        TypeRegistration::<Self>::get_or_init_with(|| TypeRegistration::<Self>::with_shape(&SHAPE, None, &[]))
    }
}

static NODE_EXPRESSION: ExpressionData = ExpressionData::Named {
    path: MUTUAL_NODE_NAME,
    definition: None,
    arguments: &[],
};
static BOX_NODE_ARGUMENTS: [GenericExpressionArgument; 1] = [GenericExpressionArgument::Type(&NODE_EXPRESSION)];

type LegacyResult = Result<(Option<ModelRole>, Vec<TypeId>), ModelMetadataError>;

fn reflection_context() -> ReflectionContext {
    ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("linked reflection context")
}

fn descriptor_in<T: ReflectTrait>(reflection: &ReflectionContext) -> TypeDescriptor<'_> {
    reflection
        .find_type_by_id(TypeId::of::<T>())
        .expect("type lookup succeeds")
        .expect("descriptor is included in the reflection context")
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

fn abi_failure_reflection() -> ReflectionContext {
    ReflectionContext::builder()
        .include_linked()
        .include_type::<MutualCycleRoot>()
        .include_type::<MutualCycleNode>()
        .include_type::<Box<MutualCycleNode>>()
        .register(CapabilityRegistration::for_type::<
            AbiFailureTarget,
            crate::__private::ModelMetadataCapability,
        >(wrong_abi_metadata_provider))
        .build()
        .expect("mismatched metadata provider is deferred until lookup")
}

fn mutual_cycle_reflection() -> ReflectionContext {
    ReflectionContext::builder()
        .include_linked()
        .include_type::<MutualCycleRoot>()
        .include_type::<MutualCycleNode>()
        .include_type::<Box<MutualCycleNode>>()
        .build()
        .expect("mutually recursive raw descriptors are registered")
}

fn previous_algorithm<'ctx>(descriptor: TypeDescriptor<'ctx>, context: &ResolutionContext<'ctx>) -> LegacyResult {
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

    let result =
        forbidden_entity_nested_role_with_visit(TypeDescriptor::of::<SharedLevel1>(), &resolution, |type_id| {
            visits.push(type_id)
        })
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
            forbidden_entity_nested_role_with_visit(TypeDescriptor::of::<SharedLevel1>(), &resolution, |id| visits
                .push(id),)
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
        assert_eq!(assert_matches_previous(descriptor, &resolution), Ok(Some(role)));
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
        assert_eq!(assert_matches_previous(descriptor, &resolution), Ok(Some(expected)));
    }
}

#[test]
fn test_nested_role_preserves_error_and_role_order_after_cycles() {
    let reflection = lookup_failure_reflection();
    let models =
        ModelRegistry::from_static_metadata(&reflection, &[]).expect("metadata-only registry defers capability lookup");

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
fn test_nested_role_preserves_abi_error_order_and_cause() {
    let reflection = abi_failure_reflection();
    let models = ModelRegistry::from_static_metadata(&reflection, &[])
        .expect("metadata-only registry defers provider ABI validation");

    let descriptor = TypeDescriptor::of::<AbiThenEntity>();
    let root = TypeMetadata::of::<AbiThenEntity>();
    let roots = [root, TypeMetadata::of::<ForbiddenEntity>()];
    let resolution = ResolutionContext::new(&models, &roots);
    let actual = forbidden_entity_nested_role(descriptor, &resolution);
    let _ = assert_matches_previous(descriptor, &resolution).expect_err("ABI failure before the role must propagate");
    assert!(matches!(
        actual,
        Err(ModelMetadataError::Abi { type_id, source, .. })
            if type_id == TypeId::of::<AbiFailureTarget>()
                && source.code() == "QMM-ABI-002"
    ));

    let root = TypeMetadata::of::<EntityThenAbi>();
    let roots = [root, TypeMetadata::of::<ForbiddenEntity>()];
    let resolution = ResolutionContext::new(&models, &roots);
    assert_eq!(
        assert_matches_previous(TypeDescriptor::of::<EntityThenAbi>(), &resolution),
        Ok(Some(ModelRole::Entity)),
        "the role before the ABI failure returns immediately",
    );
}

#[test]
fn test_nested_role_mutual_cycle_keeps_sibling_role_and_abi_error_visible() {
    let reflection = mutual_cycle_reflection();
    let root_descriptor = descriptor_in::<MutualCycleRoot>(&reflection);
    assert_eq!(root_descriptor.fields().len(), 1, "root shape is registered");
    let next_field = root_descriptor.fields().iter().next().expect("root next field");
    assert_eq!(
        next_field
            .find_concrete_type()
            .expect("mutual node pointer descriptor resolves")
            .type_id(),
        TypeId::of::<Box<MutualCycleNode>>(),
    );
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let projection_root = TypeMetadata::of::<ForbiddenProjection>();
    let role_context = ResolutionContext::new(&models, &[projection_root]);
    let mut visited = Vec::new();
    let root_descriptor = descriptor_in::<MutualCycleRoot>(&reflection);
    let role_result = forbidden_entity_nested_role_with_visit(root_descriptor, &role_context, |id| visited.push(id));
    assert_eq!(
        role_result.map_err(|error| format!("{error:?}")),
        Ok(Some(ModelRole::Projection)),
        "mutual cycle traversal visits {visited:?}",
    );
    assert_eq!(
        visited,
        [
            TypeId::of::<MutualCycleRoot>(),
            TypeId::of::<Box<MutualCycleNode>>(),
            TypeId::of::<MutualCycleNode>(),
            TypeId::of::<AbiFailureTarget>(),
            TypeId::of::<ForbiddenProjection>(),
        ],
        "the back edge is skipped only after reaching the second distinct type, then its siblings are checked",
    );
    assert_eq!(
        assert_matches_previous(root_descriptor, &role_context),
        Ok(Some(ModelRole::Projection)),
    );

    let reflection = abi_failure_reflection();
    let models = ModelRegistry::from_static_metadata(&reflection, &[])
        .expect("metadata-only registry defers provider ABI validation");
    let roots = [projection_root];
    let resolution = ResolutionContext::new(&models, &roots);
    let descriptor = descriptor_in::<MutualCycleRoot>(&reflection);
    let _ = assert_matches_previous(descriptor, &resolution)
        .expect_err("the sibling after the mutual back edge must be checked");
    assert!(matches!(
        forbidden_entity_nested_role(descriptor, &resolution),
        Err(ModelMetadataError::Abi { type_id, source, .. })
            if type_id == TypeId::of::<AbiFailureTarget>()
                && source.code() == "QMM-ABI-002"
    ));
}

#[test]
fn test_nested_role_respects_opaque_and_reference_boundaries() {
    let root = TypeMetadata::of::<OpaqueAndReferenceBoundary>();
    let reflection = reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let resolution = ResolutionContext::new(&models, &[root]);
    assert_eq!(
        assert_matches_previous(TypeDescriptor::of::<OpaqueAndReferenceBoundary>(), &resolution,),
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
        (TypeDescriptor::of::<EnumPayload>(), TypeMetadata::of::<EnumPayload>()),
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
