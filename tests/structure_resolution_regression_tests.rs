// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structural discovery and role checks must not hide behind unnamed shapes.

use std::collections::BTreeMap;

use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::Projection;
use qubit_model_derive::Value;
use qubit_model_metadata::__private::ModelMetadataCapability;
use qubit_model_metadata::__private::v8;
use qubit_model_metadata::metadata::HasTypeMetadata;
use qubit_model_metadata::metadata::ModelMetadataError;
use qubit_model_metadata::metadata::ModelRole;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::registry::ModelRegistryOrigin;
use qubit_model_metadata::resolve::ModelResolutionCause;
use qubit_model_metadata::resolve::ResolveError;
use qubit_model_metadata::resolve::ResolveErrorKind;
use qubit_model_metadata::resolve::ResolveErrors;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectionContext;
use qubit_reflect::capability::CapabilityRegistration;
use qubit_reflect::capability::CapabilityTargetId;
use qubit_reflect::context::ReflectionContextBuilder;
use qubit_reflect::descriptor::Reflect as ReflectTrait;
use qubit_reflect::error::ContextIssue;
use serde::Deserialize;
use serde::Serialize;

#[Entity(id = "structure.Inner", eq, hash)]
#[derive(Ord, PartialOrd)]
struct Inner {
    #[identifier]
    id: Id,
}

#[Entity(id = "structure.Direct")]
struct DirectEntity {
    #[identifier]
    id: Id,
    inner: Inner,
}

#[Entity(id = "structure.Tuple")]
struct TupleEntity {
    #[identifier]
    id: Id,
    inner: (Inner,),
}

#[Entity(id = "structure.Container")]
struct ContainerEntity {
    #[identifier]
    id: Id,
    inner: Option<Box<(Inner,)>>,
}

#[Entity(id = "structure.MapKey")]
struct MapKeyEntity {
    #[identifier]
    id: Id,
    inner: BTreeMap<Inner, u8>,
}

#[Entity(id = "structure.MapValue")]
struct MapValueEntity {
    #[identifier]
    id: Id,
    inner: BTreeMap<String, (Inner,)>,
}

#[Model(id = "structure.Plain", eq, hash)]
struct Plain {
    name: String,
}

#[Value]
struct NamedValue {
    inner: Plain,
}

#[Value]
struct WrappedValue(Plain);

#[Value]
struct TupleValue((u8, Plain));

#[Projection(source = Inner, eq, hash)]
struct View {
    #[identifier]
    id: Id,
}

#[Entity(id = "structure.ProjectionOwner")]
struct ProjectionEntity {
    #[identifier]
    id: Id,
    inner: (View,),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[Reflect]
struct RawWrapper {
    inner: Inner,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[Reflect]
enum RawChoice {
    Empty,
    Child { inner: Inner },
}

#[Entity(id = "structure.RawStruct")]
struct RawStructEntity {
    #[identifier]
    id: Id,
    inner: RawWrapper,
}

#[Entity(id = "structure.RawEnum")]
struct RawEnumEntity {
    #[identifier]
    id: Id,
    inner: RawChoice,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[Reflect]
struct RecursiveRaw {
    next: Option<Box<RecursiveRaw>>,
    child: Inner,
}

#[Entity(id = "structure.Recursive")]
struct RecursiveEntity {
    #[identifier]
    id: Id,
    inner: RecursiveRaw,
}

#[Entity(id = "structure.Reference")]
struct ReferenceEntity {
    #[identifier]
    id: Id,
    #[reference(entity = Inner)]
    inner: Inner,
}

#[Model]
struct ExplicitReferenceWrapper {
    #[reference(entity = Inner)]
    inner: Inner,
}

#[Entity(id = "structure.ExplicitReferenceOwner")]
struct ExplicitReferenceOwner {
    #[identifier]
    id: Id,
    wrapper: ExplicitReferenceWrapper,
}

#[Model]
struct ExplicitPlainWrapper {
    inner: Inner,
}

#[Entity(id = "structure.ExplicitPlainOwner")]
struct ExplicitPlainOwner {
    #[identifier]
    id: Id,
    wrapper: ExplicitPlainWrapper,
}

#[Entity(id = "structure.ExplicitOpaqueOwner")]
struct ExplicitOpaqueOwner {
    #[identifier]
    id: Id,
    wrapper: OpaqueRoot,
}

#[Model]
struct OpaqueRoot {
    #[opaque]
    inner: RawWrapper,
}

#[Model]
struct ModelBoundaryRoot {
    child: OpaqueRoot,
}

#[Entity(id = "structure.Opaque")]
struct OpaqueEntity {
    #[identifier]
    id: Id,
    #[opaque]
    inner: Inner,
}

#[Model(eq, hash)]
struct Child {
    number: u8,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[Reflect]
struct RawChild {
    value: Child,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[Reflect]
enum RawChildChoice {
    Empty,
    Child(Child),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[Reflect]
struct RecursiveChild {
    next: Option<Box<RecursiveChild>>,
    value: Child,
}

#[Model]
struct DiscoveryRoot {
    first: RawChild,
    second: RawChildChoice,
    third: RecursiveChild,
}

#[Model]
struct StructDiscoveryRoot {
    child: RawChild,
}

#[Model]
struct EnumDiscoveryRoot {
    child: RawChildChoice,
}

#[Model]
struct CycleDiscoveryRoot {
    child: RecursiveChild,
}

#[Value]
struct LegalChild(u8);

#[Enum]
enum LegalChoice {
    Empty,
    Number(u8),
    Value(LegalChild),
}

#[Value]
struct LegalValue(Option<Vec<(LegalChild, LegalChoice)>>);

#[Value]
struct RecursiveValue {
    next: Option<Box<RecursiveValue>>,
}

#[Value]
struct EntityValue(Inner);

#[Value]
struct ProjectionValue(View);

#[Enum]
enum InvalidChoice {
    Plain(Plain),
}

#[Value]
struct InvalidEnumValue {
    child: InvalidChoice,
}

#[Value]
struct UnresolvedValue(Plain);

#[Value]
struct RawValue(RawChild);

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[Reflect]
struct PrimitiveRaw {
    number: u8,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[Reflect]
struct EmptyRaw;

#[Value]
struct PrimitiveRawValue(PrimitiveRaw);

#[Value]
struct EmptyRawValue(EmptyRaw);

#[Value]
struct NestedInvalidValue(Plain);

#[Value]
struct SiblingValue {
    first: NestedInvalidValue,
    second: NestedInvalidValue,
}

#[Model]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
struct Invalid;

#[Value]
struct BrokenValue(Invalid);

fn reflection_builder() -> ReflectionContextBuilder {
    ReflectionContext::builder()
        .include_type::<Inner>()
        .include_type::<DirectEntity>()
        .include_type::<TupleEntity>()
        .include_type::<ContainerEntity>()
        .include_type::<MapKeyEntity>()
        .include_type::<MapValueEntity>()
        .include_type::<Plain>()
        .include_type::<NamedValue>()
        .include_type::<WrappedValue>()
        .include_type::<TupleValue>()
        .include_type::<View>()
        .include_type::<ProjectionEntity>()
        .include_type::<RawWrapper>()
        .include_type::<RawChoice>()
        .include_type::<RawStructEntity>()
        .include_type::<RawEnumEntity>()
        .include_type::<RecursiveRaw>()
        .include_type::<RecursiveEntity>()
        .include_type::<ReferenceEntity>()
        .include_type::<ExplicitReferenceWrapper>()
        .include_type::<ExplicitReferenceOwner>()
        .include_type::<ExplicitPlainWrapper>()
        .include_type::<ExplicitPlainOwner>()
        .include_type::<ExplicitOpaqueOwner>()
        .include_type::<OpaqueRoot>()
        .include_type::<ModelBoundaryRoot>()
        .include_type::<OpaqueEntity>()
        .include_type::<Child>()
        .include_type::<RawChild>()
        .include_type::<RawChildChoice>()
        .include_type::<RecursiveChild>()
        .include_type::<DiscoveryRoot>()
        .include_type::<StructDiscoveryRoot>()
        .include_type::<EnumDiscoveryRoot>()
        .include_type::<CycleDiscoveryRoot>()
        .include_type::<LegalChild>()
        .include_type::<LegalChoice>()
        .include_type::<LegalValue>()
        .include_type::<RecursiveValue>()
        .include_type::<EntityValue>()
        .include_type::<ProjectionValue>()
        .include_type::<InvalidChoice>()
        .include_type::<InvalidEnumValue>()
        .include_type::<UnresolvedValue>()
        .include_type::<RawValue>()
        .include_type::<PrimitiveRaw>()
        .include_type::<EmptyRaw>()
        .include_type::<PrimitiveRawValue>()
        .include_type::<EmptyRawValue>()
        .include_type::<NestedInvalidValue>()
        .include_type::<SiblingValue>()
        .include_type::<Invalid>()
        .include_type::<BrokenValue>()
        .include_type::<ExplicitOpaqueDiscoveryRoot>()
}

fn empty_reflection() -> ReflectionContext {
    reflection_builder().build().expect("empty reflection context")
}

macro_rules! reflection_with_models {
    ($($model:ty),+ $(,)?) => {{
        let builder = reflection_builder();
        $(let builder = builder.register(v8::model_metadata_registration::<$model>());)+
        builder.build().expect("reflection context with selected model metadata")
    }};
}

fn duplicate_metadata_registration_error() -> qubit_reflect::error::ContextError {
    match reflection_builder()
        .register(v8::model_metadata_registration::<Invalid>())
        .register(v8::model_metadata_registration::<Invalid>())
        .build()
    {
        Ok(_) => panic!("duplicate typed model metadata registration is invalid"),
        Err(error) => error,
    }
}

fn invalid_metadata_provider() -> &'static TypeMetadata {
    TypeMetadata::of::<BrokenValue>()
}

fn invalid_metadata_reflection() -> ReflectionContext {
    reflection_builder()
        .register(CapabilityRegistration::for_type::<Invalid, ModelMetadataCapability>(
            invalid_metadata_provider,
        ))
        .build()
        .expect("one typed model metadata provider is valid")
}

fn recursive_value_reflection() -> ReflectionContext {
    ReflectionContext::builder()
        .include_linked_for::<RecursiveValue>()
        .register(v8::model_metadata_registration::<RecursiveValue>())
        .build()
        .expect("linked recursive value context")
}

const TEST_ORIGIN: ModelRegistryOrigin = ModelRegistryOrigin {
    crate_name: "structure-resolution-regression-tests",
    crate_version: None,
    module_path: "structure_resolution_regression_tests",
    file: Some(file!()),
    line: Some(1),
    column: Some(1),
};

#[Model]
struct ExplicitOpaqueDiscoveryRoot {
    #[opaque]
    hidden: RawChild,
}

/// Resolves only one explicit root against a fresh, empty reflection snapshot.
fn root_errors<T: HasTypeMetadata + ReflectTrait + 'static>() -> ResolveErrors {
    let root = TypeMetadata::of::<T>();
    // These are the only nested model roles used by the generic rejection
    // fixtures. Register them explicitly in the same test-local context.
    let reflection = reflection_with_models!(T, Inner, View, Plain);
    root_errors_in_context(root, &reflection)
}

fn root_errors_without_named_child() -> ResolveErrors {
    let root = TypeMetadata::of::<UnresolvedValue>();
    let reflection = reflection_with_models!(UnresolvedValue);
    root_errors_in_context(root, &reflection)
}

fn root_errors_in_context(root: &'static TypeMetadata, reflection: &ReflectionContext) -> ResolveErrors {
    let models = ModelRegistry::from_context(reflection).expect("model metadata registration remains valid");
    let roots = [root];
    match StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    {
        Err(errors) => errors,
        Ok(_) => panic!("invalid structure must fail for {}", root.type_name()),
    }
}

/// Selects the expected root diagnostic and checks its concrete declaration.
fn root_error<'errors>(
    errors: &'errors ResolveErrors,
    root: &'static TypeMetadata,
    kind: ResolveErrorKind,
    field: usize,
) -> &'errors ResolveError {
    let error = errors
        .errors()
        .iter()
        .find(|error| error.kind() == kind && error.owner_type_id() == Some(root.type_id()))
        .expect("root error with exact kind and owner");
    assert_eq!(error.owner_type_name(), Some(root.type_name()));
    let declaration = error.declaration().expect("exact declaration retained");
    assert_eq!(declaration.field, Some(field));
    assert_eq!(error.owner_type_name(), Some(root.type_name()));
    error
}

#[test]
fn test_entity_tuple_matches_direct_rejection() {
    for (root, resolve_errors) in [
        (
            TypeMetadata::of::<DirectEntity>(),
            root_errors::<DirectEntity> as fn() -> ResolveErrors,
        ),
        (TypeMetadata::of::<TupleEntity>(), root_errors::<TupleEntity>),
    ] {
        let errors = resolve_errors();
        let error = root_error(&errors, root, ResolveErrorKind::InvalidEntityNesting, 1);
        let declaration = error.declaration().expect("owning Entity field");
        assert_eq!(error.owner_type_name(), Some(root.type_name()));
        assert_eq!(declaration.variant, None);
        assert_eq!(error.actual_role(), Some(ModelRole::Entity));
        assert_eq!(error.path().expect("named owning field").segments(), &["inner"]);
    }
}

#[test]
fn test_value_newtype_matches_named_rejection() {
    for (root, resolve_errors, field, path) in [
        (
            TypeMetadata::of::<NamedValue>(),
            root_errors::<NamedValue> as fn() -> ResolveErrors,
            0,
            Some("inner"),
        ),
        (TypeMetadata::of::<WrappedValue>(), root_errors::<WrappedValue>, 0, None),
        (TypeMetadata::of::<TupleValue>(), root_errors::<TupleValue>, 0, None),
    ] {
        let errors = resolve_errors();
        let error = root_error(&errors, root, ResolveErrorKind::InvalidValueClosure, field);
        assert_eq!(error.actual_role(), Some(ModelRole::Model));
        assert_eq!(error.expected_role(), Some(ModelRole::Value));
        assert_eq!(error.path().map(|path| path.to_string()), path.map(str::to_owned));
    }
}

#[test]
fn test_entity_nested_wrappers_preserve_owning_field() {
    for (root, resolve_errors, role) in [
        (
            TypeMetadata::of::<ContainerEntity>(),
            root_errors::<ContainerEntity> as fn() -> ResolveErrors,
            ModelRole::Entity,
        ),
        (
            TypeMetadata::of::<MapKeyEntity>(),
            root_errors::<MapKeyEntity>,
            ModelRole::Entity,
        ),
        (
            TypeMetadata::of::<MapValueEntity>(),
            root_errors::<MapValueEntity>,
            ModelRole::Entity,
        ),
        (
            TypeMetadata::of::<RawStructEntity>(),
            root_errors::<RawStructEntity>,
            ModelRole::Entity,
        ),
        (
            TypeMetadata::of::<RawEnumEntity>(),
            root_errors::<RawEnumEntity>,
            ModelRole::Entity,
        ),
        (
            TypeMetadata::of::<ProjectionEntity>(),
            root_errors::<ProjectionEntity>,
            ModelRole::Projection,
        ),
        (
            TypeMetadata::of::<RecursiveEntity>(),
            root_errors::<RecursiveEntity>,
            ModelRole::Entity,
        ),
    ] {
        let errors = resolve_errors();
        let error = root_error(&errors, root, ResolveErrorKind::InvalidEntityNesting, 1);
        let declaration = error.declaration().expect("owning Entity field");
        assert_eq!(error.owner_type_name(), Some(root.type_name()));
        assert_eq!(declaration.variant, None);
        assert_eq!(error.actual_role(), Some(role));
        assert_eq!(error.path().expect("outer field path").segments(), &["inner"]);
    }
}

#[test]
fn test_only_root_discovers_raw_struct_enum_and_recursive_children_once() {
    for root in [
        TypeMetadata::of::<DiscoveryRoot>(),
        TypeMetadata::of::<StructDiscoveryRoot>(),
        TypeMetadata::of::<EnumDiscoveryRoot>(),
        TypeMetadata::of::<CycleDiscoveryRoot>(),
    ] {
        let reflection = reflection_with_models!(Child);
        let models = ModelRegistry::from_context(&reflection).expect("empty model registry");
        let roots = [root];
        let graph = StructureResolver::new(ResolveInputs {
            models: &models,
            roots: &roots,
        })
        .resolve()
        .expect("ordinary raw wrappers are discoverable");
        let child = TypeMetadata::of::<Child>();
        assert!(graph.properties(child).is_some(), "reachable child is a graph node");
        assert_eq!(
            graph
                .models()
                .iter()
                .filter(|node| node.type_id() == child.type_id())
                .count(),
            1
        );
        assert_eq!(graph.models().len(), 2);
        assert!(
            models.by_type_id(TypeMetadata::of::<Plain>().type_id()).is_none(),
            "the test-local context did not import unrelated model registrations"
        );
        let metadata_only_reflection = empty_reflection();
        let metadata_only = ModelRegistry::from_static_metadata(&metadata_only_reflection, &[])
            .expect("metadata-only comparison registry");
        let graph = StructureResolver::new(ResolveInputs {
            models: &metadata_only,
            roots: &roots,
        })
        .resolve()
        .expect("metadata-only snapshot resolves its explicit root");
        assert!(
            graph.model(child.type_id()).is_none(),
            "same-shape metadata-only graph cannot import Child capabilities"
        );
        assert_eq!(graph.models().len(), 1);
    }
}

#[test]
fn test_legal_value_and_reference_contrasts() {
    for root in [
        TypeMetadata::of::<LegalValue>(),
        TypeMetadata::of::<RecursiveValue>(),
        TypeMetadata::of::<ReferenceEntity>(),
    ] {
        let reflection = if root.type_id() == TypeMetadata::of::<RecursiveValue>().type_id() {
            recursive_value_reflection()
        } else {
            reflection_with_models!(LegalValue, ReferenceEntity, LegalChoice, LegalChild, Inner, Plain)
        };
        let models = ModelRegistry::from_context(&reflection).expect("empty model registry");
        let _graph = StructureResolver::new(ResolveInputs {
            models: &models,
            roots: &[root],
        })
        .resolve()
        .expect("legal closed values and explicit references remain valid");
    }
}

#[test]
fn test_opaque_model_checks_and_discovery_boundary() {
    let root = TypeMetadata::of::<OpaqueEntity>();
    assert_eq!(
        root.field("inner").expect("opaque field").opaque_type_id(),
        Some(std::any::TypeId::of::<Inner>()),
        "opaque metadata retains exact static field identity without Reflect"
    );
    let reflection = reflection_with_models!(Inner, OpaqueEntity);
    let isolated = ModelRegistry::from_context(&reflection).expect("only relevant models are registered");
    let errors = StructureResolver::new(ResolveInputs {
        models: &isolated,
        roots: &[root],
    })
    .resolve()
    .expect_err("opaque registered Entity remains rejected");
    root_error(&errors, root, ResolveErrorKind::OpaqueModel, 1);
    let reflection = reflection_with_models!(ModelBoundaryRoot, OpaqueRoot);
    let models = ModelRegistry::from_context(&reflection).expect("empty model registry");
    let roots = [TypeMetadata::of::<OpaqueRoot>()];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("opaque non-model wrapper stays opaque");
    assert_eq!(graph.models().len(), 1, "opaque wrapper must not expose Inner");
    let roots = [TypeMetadata::of::<ModelBoundaryRoot>()];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("nested model keeps metadata opaque boundary");
    assert_eq!(graph.models().len(), 2);
    assert!(graph.model(TypeMetadata::of::<Inner>().type_id()).is_none());
}

#[test]
fn test_raw_struct_is_not_automatically_a_closed_value() {
    let root = TypeMetadata::of::<RawValue>();
    let errors = root_errors::<RawValue>();
    root_error(&errors, root, ResolveErrorKind::InvalidValueClosure, 0);
}

#[test]
fn test_primitive_only_and_empty_raw_structs_are_not_closed_values() {
    for (root, resolve_errors) in [
        (
            TypeMetadata::of::<PrimitiveRawValue>(),
            root_errors::<PrimitiveRawValue> as fn() -> ResolveErrors,
        ),
        (TypeMetadata::of::<EmptyRawValue>(), root_errors::<EmptyRawValue>),
    ] {
        let errors = resolve_errors();
        let error = root_error(&errors, root, ResolveErrorKind::InvalidValueClosure, 0);
        assert_eq!(
            error.actual_role(),
            None,
            "the raw struct itself has no legal Value role"
        );
        assert!(error.path().is_none());
        assert_eq!(error.owner_type_id(), Some(root.type_id()));
        assert_eq!(error.owner_type_name(), Some(root.type_name()));
    }
    let reflection = reflection_with_models!(LegalChild, LegalValue, LegalChoice, Inner, Plain);
    let models = ModelRegistry::from_context(&reflection).expect("registered legal value metadata");
    let roots = [TypeMetadata::of::<LegalChild>(), TypeMetadata::of::<LegalValue>()];
    let _graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("primitive newtypes and nested Value/Enum combinations remain legal");
}

#[test]
fn test_sibling_value_errors_keep_both_paths_and_nearest_declaration() {
    let root = TypeMetadata::of::<SiblingValue>();
    let reflection = reflection_with_models!(SiblingValue, NestedInvalidValue, Plain);
    let errors = root_errors_in_context(root, &reflection);
    let errors: Vec<_> = errors
        .errors()
        .iter()
        .filter(|error| {
            error.kind() == ResolveErrorKind::InvalidValueClosure && error.owner_type_id() == Some(root.type_id())
        })
        .collect();
    assert_eq!(errors.len(), 2, "sibling uses must both be checked");
    let paths: Vec<_> = errors
        .iter()
        .map(|error| error.path().expect("sibling path").to_string())
        .collect();
    assert_eq!(paths, ["first", "second"]);
    for error in errors {
        let declaration = error.declaration().expect("unnamed nested declaration");
        assert_eq!(
            declaration.owner, None,
            "static declaration facts do not bind owner names"
        );
        assert_eq!(declaration.field, Some(0));
    }
}

#[test]
fn test_unnamed_value_role_failure_has_no_fake_path() {
    let root = TypeMetadata::of::<BrokenValue>();
    let reflection = reflection_with_models!(Invalid);
    let errors = root_errors_in_context(root, &reflection);
    let error = root_error(&errors, root, ResolveErrorKind::InvalidValueClosure, 0);
    assert!(
        error.path().is_none(),
        "unnamed fields never acquire a fake property name"
    );
    assert_eq!(error.actual_role(), Some(ModelRole::Model));
    assert_eq!(error.owner_type_id(), Some(root.type_id()));
    assert_eq!(error.declaration().expect("unnamed field source").field, Some(0));
}

#[test]
fn test_model_registry_preserves_wrong_typed_metadata_provider_error() {
    let reflection = invalid_metadata_reflection();
    let error = ModelRegistry::from_context(&reflection)
        .expect_err("registry rejects a provider that returns another type's metadata");
    assert_eq!(
        error.kind(),
        qubit_model_metadata::registry::ModelRegistryErrorKind::InvalidMetadata
    );
    assert_eq!(error.abi_cause().expect("typed ABI cause").code(), "QMM-ABI-002");
    assert_eq!(
        error.sources()[0].module_path,
        "structure_resolution_regression_tests::Invalid"
    );
}

#[test]
fn test_value_rejects_entity_projection_and_unresolved_newtypes() {
    for (root, resolve_errors, role) in [
        (
            TypeMetadata::of::<EntityValue>(),
            root_errors::<EntityValue> as fn() -> ResolveErrors,
            Some(ModelRole::Entity),
        ),
        (
            TypeMetadata::of::<ProjectionValue>(),
            root_errors::<ProjectionValue>,
            Some(ModelRole::Projection),
        ),
        (
            TypeMetadata::of::<UnresolvedValue>(),
            root_errors_without_named_child,
            None,
        ),
    ] {
        let errors = resolve_errors();
        let error = root_error(&errors, root, ResolveErrorKind::InvalidValueClosure, 0);
        assert_eq!(error.actual_role(), role);
        assert!(error.path().is_none());
    }
}

#[test]
fn test_value_enum_failure_keeps_nearest_payload_declaration() {
    let root = TypeMetadata::of::<InvalidEnumValue>();
    let reflection = reflection_with_models!(InvalidEnumValue, InvalidChoice, Plain);
    let errors = root_errors_in_context(root, &reflection);
    let error = root_error(&errors, root, ResolveErrorKind::InvalidValueClosure, 0);
    let declaration = error.declaration().expect("enum payload declaration");
    assert_eq!(
        declaration.owner, None,
        "static declaration facts do not bind owner names"
    );
    assert_eq!(declaration.variant, Some(0));
    assert_eq!(error.actual_role(), Some(ModelRole::Model));
    assert_eq!(error.path().expect("available enum path").to_string(), "child.PLAIN");
}

/// Exercises explicit roots with metadata-only and reflection-backed
/// registries.
fn check_explicit_roots(include_inner: bool) {
    let reflection = empty_reflection();
    let models = if include_inner {
        ModelRegistry::from_static_metadata(&reflection, &[(TypeMetadata::of::<Inner>(), &TEST_ORIGIN)])
            .expect("metadata-only Entity target registry")
    } else {
        ModelRegistry::from_context(&reflection).expect("empty model registry")
    };
    let root = TypeMetadata::of::<ExplicitReferenceOwner>();
    let wrapper = TypeMetadata::of::<ExplicitReferenceWrapper>();
    let roots = [root, wrapper];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("explicit anonymous wrapper retains its reference boundary");
    assert!(graph.properties(wrapper).is_some());
    assert_eq!(
        graph.models().len(),
        3,
        "only owner, wrapper, and registered reference target are nodes"
    );
    assert!(
        graph
            .reference(
                wrapper.fields()[0]
                    .location()
                    .expect("concrete wrapper reference field")
            )
            .is_some()
    );

    let plain_reflection = reflection_with_models!(ExplicitPlainOwner, ExplicitPlainWrapper, Inner);
    let plain_models = ModelRegistry::from_context(&plain_reflection).expect("plain wrapper model context");
    let root = TypeMetadata::of::<ExplicitPlainOwner>();
    let roots = [root, TypeMetadata::of::<ExplicitPlainWrapper>()];
    let errors = StructureResolver::new(ResolveInputs {
        models: &plain_models,
        roots: &roots,
    })
    .resolve()
    .expect_err("non-reference Entity nesting remains invalid");
    let error = root_error(&errors, root, ResolveErrorKind::InvalidEntityNesting, 1);
    assert_eq!(error.actual_role(), Some(ModelRole::Entity));
    assert_eq!(error.path().expect("owning wrapper path").segments(), &["wrapper"]);
    assert_eq!(error.owner_type_id(), Some(root.type_id()));

    let roots = [
        TypeMetadata::of::<ExplicitOpaqueOwner>(),
        TypeMetadata::of::<OpaqueRoot>(),
    ];
    let _ = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("opaque raw field stops traversal inside an explicit wrapper");
}

#[test]
fn test_explicit_anonymous_roots_preserve_reference_and_opaque_boundaries() {
    check_explicit_roots(true);
    check_explicit_roots(false);
}

#[test]
fn test_explicit_only_value_and_enum_roots_are_closed() {
    let roots = [
        TypeMetadata::of::<LegalValue>(),
        TypeMetadata::of::<LegalChoice>(),
        TypeMetadata::of::<LegalChild>(),
    ];
    let reflection = empty_reflection();
    let models = ModelRegistry::from_static_metadata(&reflection, &[]).expect("empty metadata-only registry");
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("explicit Value and Enum roots supply closed child metadata");
    for root in roots {
        assert!(graph.properties(root).is_some());
    }
    let reflection = empty_reflection();
    let models = ModelRegistry::from_context(&reflection).expect("empty model registry");
    let _ = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("reflection-backed equivalent remains closed");
}

#[test]
fn test_duplicate_model_metadata_capability_is_rejected_before_resolution() {
    let error = duplicate_metadata_registration_error();
    assert!(matches!(
        error.primary_issue(),
        ContextIssue::ConflictingCapability {
            target: CapabilityTargetId::Type(identity),
            capability,
            ..
        } if identity.type_id == std::any::TypeId::of::<Invalid>()
            && capability.stable_id.0.as_ref() == "qubit.model.metadata.v1"
    ));
}

#[test]
fn test_explicit_opaque_root_does_not_discover_hidden_model() {
    let roots = [TypeMetadata::of::<ExplicitOpaqueDiscoveryRoot>()];
    let reflection = empty_reflection();
    let models = ModelRegistry::from_static_metadata(&reflection, &[]).expect("empty metadata-only registry");
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("opaque explicit root");
    assert_eq!(graph.models().len(), 1);
    assert!(graph.model(TypeMetadata::of::<Child>().type_id()).is_none());
}

#[test]
fn test_explicit_roots_fallback_validates_descriptor_abi() {
    // Omitting the reflected field overlay deliberately creates a malformed
    // explicit input through the same unchecked ABI used by generated code.
    let malformed = v8::leak(
        v8::GeneratedTypeMetadataBuilder::new::<PrimitiveRaw>(None, &[], v8::leak(v8::model_role())).finish_unchecked(),
    );
    let roots = [TypeMetadata::of::<PrimitiveRawValue>(), malformed];
    let reflection = empty_reflection();
    let models = ModelRegistry::from_static_metadata(&reflection, &[]).expect("empty metadata-only registry");
    for resolver in [StructureResolver::new, StructureResolver::for_roots] {
        let errors = resolver(ResolveInputs {
            models: &models,
            roots: &roots,
        })
        .resolve()
        .expect_err("malformed explicit metadata must satisfy descriptor ABI");
        let error = errors
            .errors()
            .iter()
            .find(|error| {
                error.kind() == ResolveErrorKind::MetadataResolution
                    && error.owner_type_id() == Some(malformed.type_id())
            })
            .expect("ABI failure identifies the malformed explicit root");
        let Some(ModelResolutionCause::Metadata(ModelMetadataError::Abi { type_id, source, .. })) = error.cause()
        else {
            panic!("explicit root must retain its typed ABI failure");
        };
        assert_eq!(*type_id, malformed.type_id());
        assert_eq!(source.code(), "QMM-ABI-003");
        assert!(error.path().is_none());
        assert!(
            !errors
                .errors()
                .iter()
                .any(|error| error.kind() == ResolveErrorKind::InvalidValueClosure)
        );
    }
}

#[test]
fn test_for_roots_resolves_only_the_reachable_registered_subgraph() {
    let reflection = empty_reflection();
    let models = ModelRegistry::from_static_metadata(
        &reflection,
        &[
            (TypeMetadata::of::<Inner>(), &TEST_ORIGIN),
            (TypeMetadata::of::<DirectEntity>(), &TEST_ORIGIN),
        ],
    )
    .expect("valid and unrelated concrete registrations");
    let root = TypeMetadata::of::<ExplicitReferenceWrapper>();
    let roots = [root];
    let inputs = ResolveInputs {
        models: &models,
        roots: &roots,
    };

    let full_errors = StructureResolver::new(inputs)
        .resolve()
        .expect_err("complete audit sees the unrelated invalid entity nesting");
    assert!(
        full_errors
            .errors()
            .iter()
            .any(|error| error.kind() == ResolveErrorKind::InvalidEntityNesting
                && error.owner_type_id() == Some(TypeMetadata::of::<DirectEntity>().type_id()))
    );

    let graph = StructureResolver::for_roots(inputs)
        .resolve()
        .expect("local resolution skips unrelated invalid registrations");
    assert!(graph.model(root.type_id()).is_some());
    assert!(graph.model(TypeMetadata::of::<Inner>().type_id()).is_some());
    assert!(graph.model(TypeMetadata::of::<DirectEntity>().type_id()).is_none());
    assert_eq!(graph.models().len(), 2);
    assert!(graph.properties(TypeMetadata::of::<DirectEntity>()).is_none());
}

#[test]
fn test_for_roots_empty_roots_produce_an_empty_graph() {
    let reflection = empty_reflection();
    let models = ModelRegistry::from_static_metadata(&reflection, &[(TypeMetadata::of::<Inner>(), &TEST_ORIGIN)])
        .expect("registered concrete model");
    let roots: [&'static TypeMetadata; 0] = [];
    let inputs = ResolveInputs {
        models: &models,
        roots: &roots,
    };

    let local = StructureResolver::for_roots(inputs)
        .resolve()
        .expect("empty local root set");
    assert!(local.models().is_empty());

    let complete = StructureResolver::new(inputs)
        .resolve()
        .expect("empty-root complete audit");
    assert_eq!(complete.models().len(), 1);
    assert!(complete.model(TypeMetadata::of::<Inner>().type_id()).is_some());
}

#[test]
fn test_for_roots_rejects_malformed_overlay_even_when_registry_has_canonical_metadata() {
    let registered = TypeMetadata::of::<Plain>();
    let reflection = empty_reflection();
    let models = ModelRegistry::from_static_metadata(&reflection, &[(registered, &TEST_ORIGIN)])
        .expect("valid canonical model metadata");
    let malformed = v8::leak(
        v8::GeneratedTypeMetadataBuilder::new::<Plain>(None, &[], v8::leak(v8::model_role())).finish_unchecked(),
    );
    let roots = [malformed];
    let inputs = ResolveInputs {
        models: &models,
        roots: &roots,
    };

    for resolver in [StructureResolver::new, StructureResolver::for_roots] {
        let errors = resolver(inputs)
            .resolve()
            .expect_err("malformed explicit overlay must not be replaced silently");
        assert!(errors.errors().iter().any(|error| {
            error.kind() == ResolveErrorKind::MetadataResolution
                && error.owner_type_id() == Some(malformed.type_id())
                && matches!(
                    error.cause(),
                    Some(ModelResolutionCause::Metadata(ModelMetadataError::Abi { source, .. }))
                        if source.code() == "QMM-ABI-003"
                )
        }));
    }
}

#[test]
fn test_for_roots_deduplicates_repeated_roots() {
    let metadata = TypeMetadata::of::<Plain>();
    let roots = [metadata, metadata];
    let reflection = empty_reflection();
    let models = ModelRegistry::from_static_metadata(&reflection, &[]).expect("empty registry");
    let graph = StructureResolver::for_roots(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("duplicate valid root");

    assert_eq!(graph.models().len(), 1);
    assert_eq!(graph.models()[0].type_id(), metadata.type_id());
}

#[test]
fn test_for_roots_reports_all_malformed_roots_in_deterministic_order() {
    let malformed_primitive = v8::leak(
        v8::GeneratedTypeMetadataBuilder::new::<PrimitiveRaw>(None, &[], v8::leak(v8::model_role())).finish_unchecked(),
    );
    let malformed_plain = v8::leak(
        v8::GeneratedTypeMetadataBuilder::new::<Plain>(None, &[], v8::leak(v8::model_role())).finish_unchecked(),
    );
    let roots = [malformed_primitive, malformed_plain];
    let reflection = empty_reflection();
    let models = ModelRegistry::from_static_metadata(&reflection, &[]).expect("empty registry");
    let errors = StructureResolver::for_roots(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect_err("both malformed roots must be reported");

    assert_eq!(errors.errors().len(), 2);
    assert_eq!(
        errors.errors()[0].owner_type_name(),
        Some("structure_resolution_regression_tests::Plain")
    );
    assert_eq!(
        errors.errors()[1].owner_type_name(),
        Some("structure_resolution_regression_tests::PrimitiveRaw")
    );
    assert!(errors.errors().iter().all(|error| matches!(
        error.cause(),
        Some(ModelResolutionCause::Metadata(ModelMetadataError::Abi { source, .. }))
            if source.code() == "QMM-ABI-003"
    )));
}
