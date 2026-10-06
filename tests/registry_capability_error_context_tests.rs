// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Verifies that downstream users can inspect capability conflict context.

use std::any::TypeId;
#[cfg(feature = "generic")]
use std::sync::LazyLock;

use qubit_model_metadata::__private::ModelImplProvider;
use qubit_model_metadata::__private::ModelMetadataProvider;
use qubit_model_metadata::__private::model_impl_key;
use qubit_model_metadata::__private::model_metadata_key;
use qubit_model_metadata::__private::register_type_capabilities;
#[cfg(feature = "generic")]
use qubit_model_metadata::__private::v7;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::registry::ModelRegistryErrorKind;
use qubit_reflect::Reflect;
#[cfg(feature = "generic")]
use qubit_reflect::TypeDefinitionDescriptor;
#[cfg(feature = "generic")]
use qubit_reflect::TypeDefinitionId;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::capability::CapabilityConflict;
use qubit_reflect::capability::CapabilityConflictKind;
use qubit_reflect::capability::CapabilityDescriptor;
use qubit_reflect::capability::CapabilityKey;
use qubit_reflect::capability::CapabilityOrigin;
use qubit_reflect::error::RegistryError;
use qubit_reflect::error::RegistryErrorKind;
#[cfg(feature = "generic")]
use qubit_reflect::expression::GenericDefinitionDescriptor;
use qubit_reflect::identity::CapabilityId;
use qubit_reflect::identity::FragmentIdentity;
use qubit_reflect::register_reflected_type;
use qubit_reflect::registry::CapabilityTarget;
use qubit_reflect::registry::ReflectRegistry;
use qubit_reflect::registry::RegistrySnapshotBuilder;

#[derive(Reflect)]
#[reflect(crate = qubit_reflect)]
struct DiagnosticsTarget;

#[derive(Reflect)]
#[reflect(crate = qubit_reflect)]
struct GlobalConflict;

#[cfg(feature = "generic")]
struct GenericOrphan;

#[cfg(feature = "generic")]
static EMPTY_GENERICS: LazyLock<GenericDefinitionDescriptor> =
    LazyLock::new(|| GenericDefinitionDescriptor::new([], []));
#[cfg(feature = "generic")]
static GENERIC_ORPHAN_DEFINITION: LazyLock<TypeDefinitionDescriptor> = LazyLock::new(|| {
    TypeDefinitionDescriptor::enum_type(
        TypeDefinitionId::of::<GenericOrphan>(),
        "snapshot::GenericOrphan",
        "GenericOrphan",
        &EMPTY_GENERICS,
        &[],
    )
});

register_reflected_type!(GlobalConflict);
register_type_capabilities!(GlobalConflict: [key("model.test.global_context") => 11_u32]);
register_type_capabilities!(GlobalConflict: [key("model.test.global_context") => 13_u64]);

fn key<A: 'static>(id: &'static str) -> CapabilityKey<A> {
    CapabilityKey::new(CapabilityId::new(id).expect("valid fixture capability ID"))
}

fn source(declaring_crate: &'static str, line: u32) -> FragmentIdentity {
    FragmentIdentity::new(
        declaring_crate,
        "registry_capability_error_context_tests",
        line,
        1,
        "capability",
        u64::from(line),
    )
}

fn model_provider_snapshot(capability: CapabilityDescriptor) -> (ReflectRegistry, FragmentIdentity) {
    let target = TypeDescriptor::of::<DiagnosticsTarget>();
    let type_source = source("model-provider-type", 50);
    let mut builder = RegistrySnapshotBuilder::new();
    builder.add_type_with_capabilities(
        target,
        vec![capability],
        type_source.clone(),
        source("model-provider-capability", 51),
    );
    (
        builder.build().expect("valid isolated snapshot"),
        source("model-provider-capability", 51),
    )
}

fn unreachable_metadata_provider() -> &'static qubit_model_metadata::metadata::TypeMetadata {
    panic!("orphan audit must precede provider invocation")
}

fn unreachable_model_impl_provider() -> &'static qubit_model_metadata::metadata::ModelImplMetadata {
    panic!("unrelated ModelImpl provider is not projected here")
}

#[cfg(feature = "generic")]
fn unreachable_generic_metadata_provider() -> &'static qubit_model_metadata::generic::GenericModelMetadata {
    panic!("orphan audit must precede generic provider invocation")
}

#[cfg(feature = "generic")]
#[test]
fn test_model_registry_rejects_capability_only_generic_targets_before_provider_call() {
    let capability_source = source("orphan-generic-capability", 70);
    let mut builder = RegistrySnapshotBuilder::new();
    builder.add_definition_capabilities(
        &GENERIC_ORPHAN_DEFINITION,
        vec![CapabilityDescriptor::with_adapter(
            v7::generic_model_metadata_key(),
            unreachable_generic_metadata_provider
                as fn() -> &'static qubit_model_metadata::generic::GenericModelMetadata,
        )],
        capability_source.clone(),
    );
    let reflection = builder.build().expect("capability-only generic fact is valid");
    let error = ModelRegistry::from_reflect_registry(&reflection).expect_err("orphan generic target rejected");
    assert_eq!(error.kind(), ModelRegistryErrorKind::UnregisteredModelTarget);
    assert_eq!(
        error.capability_target(),
        Some(CapabilityTarget::TypeDefinition(GENERIC_ORPHAN_DEFINITION.id()))
    );
    assert_eq!(error.capability_id(), Some(*v7::generic_model_metadata_key().id()));
    assert_eq!(error.sources(), &[capability_source]);
}

#[cfg(feature = "generic")]
#[test]
fn test_generic_provider_contract_errors_report_capability_sources() {
    for capability in [
        CapabilityDescriptor::without_adapter(v7::generic_model_metadata_key()),
        CapabilityDescriptor::with_adapter(key::<u32>("qubit.model.generic_metadata.v1"), 7_u32),
    ] {
        let declaration_source = source("generic-model-declaration", 71);
        let capability_source = source("generic-model-capability", 72);
        let mut builder = RegistrySnapshotBuilder::new();
        builder.add_definition(&GENERIC_ORPHAN_DEFINITION, declaration_source);
        builder.add_definition_capabilities(&GENERIC_ORPHAN_DEFINITION, vec![capability], capability_source.clone());
        let reflection = builder.build().expect("valid generic capability snapshot");
        let error = ModelRegistry::from_reflect_registry(&reflection).expect_err("invalid provider contract");
        assert!(matches!(
            error.kind(),
            ModelRegistryErrorKind::FactOnlyCapability | ModelRegistryErrorKind::AdapterTypeMismatch
        ));
        assert_eq!(error.sources(), &[capability_source]);
        assert_eq!(error.origins().len(), 1);
    }
}

#[test]
fn test_model_registry_rejects_capability_only_model_targets_before_provider_call() {
    let orphan_source = source("orphan-model-capability", 61);
    let mut builder = RegistrySnapshotBuilder::new();
    builder.add_type_capabilities(
        TypeDescriptor::of::<DiagnosticsTarget>(),
        vec![CapabilityDescriptor::with_adapter(
            model_metadata_key(),
            unreachable_metadata_provider as ModelMetadataProvider,
        )],
        orphan_source.clone(),
    );
    let reflection = builder.build().expect("capability-only fact is a valid snapshot");
    let error = ModelRegistry::from_reflect_registry(&reflection).expect_err("orphan model capability rejected");
    assert_eq!(error.kind(), ModelRegistryErrorKind::UnregisteredModelTarget);
    assert_eq!(
        error.capability_target(),
        Some(CapabilityTarget::Type(TypeId::of::<DiagnosticsTarget>()))
    );
    assert_eq!(error.capability_id(), Some(*model_metadata_key().id()));
    assert_eq!(error.sources(), &[orphan_source]);

    for capability in [
        CapabilityDescriptor::without_adapter(model_metadata_key()),
        CapabilityDescriptor::with_adapter(key::<u32>("qubit.model.metadata.v1"), 7_u32),
    ] {
        let mut builder = RegistrySnapshotBuilder::new();
        builder.add_type_capabilities(
            TypeDescriptor::of::<DiagnosticsTarget>(),
            vec![capability],
            source("orphan-model-capability-variant", 62),
        );
        let reflection = builder.build().expect("capability-only fact is valid");
        let error = ModelRegistry::from_reflect_registry(&reflection).expect_err("orphan model target rejected");
        assert_eq!(error.kind(), ModelRegistryErrorKind::UnregisteredModelTarget);
        assert_eq!(
            error.capability_target(),
            Some(CapabilityTarget::Type(TypeId::of::<DiagnosticsTarget>()))
        );
    }

    let mut builder = RegistrySnapshotBuilder::new();
    builder.add_type_capabilities(
        TypeDescriptor::of::<DiagnosticsTarget>(),
        vec![CapabilityDescriptor::with_adapter(
            model_impl_key(),
            unreachable_model_impl_provider as ModelImplProvider,
        )],
        source("orphan-model-impl-capability", 63),
    );
    let reflection = builder.build().expect("ModelImpl capability is independent");
    assert!(
        ModelRegistry::from_reflect_registry(&reflection)
            .expect("ModelImpl-only capability does not create a model requirement")
            .entries()
            .is_empty()
    );
}

#[test]
fn test_model_registry_rejects_fact_only_model_provider() {
    let (reflection, expected_source) =
        model_provider_snapshot(CapabilityDescriptor::without_adapter(model_metadata_key()));

    let error =
        ModelRegistry::from_reflect_registry(&reflection).expect_err("a model capability without a provider must fail");

    assert_eq!(error.kind(), ModelRegistryErrorKind::FactOnlyCapability);
    assert_eq!(error.capability_id(), Some(*model_metadata_key().id()));
    assert_eq!(error.expected_adapter_type(), None);
    assert_eq!(error.actual_adapter_type(), None);
    assert_eq!(error.sources(), std::slice::from_ref(&expected_source));
    assert_eq!(
        error.origins(),
        &[CapabilityOrigin::Registered {
            source: expected_source
        }]
    );
}

#[test]
fn test_model_registry_rejects_model_provider_with_wrong_adapter_type() {
    let wrong_key = key::<u32>("qubit.model.metadata.v1");
    let (reflection, expected_source) = model_provider_snapshot(CapabilityDescriptor::with_adapter(wrong_key, 7_u32));

    let error = ModelRegistry::from_reflect_registry(&reflection)
        .expect_err("a model capability with the wrong provider type must fail");

    assert_eq!(error.kind(), ModelRegistryErrorKind::AdapterTypeMismatch);
    assert_eq!(error.capability_id(), Some(*model_metadata_key().id()));
    assert_eq!(
        error.expected_adapter_type(),
        Some(TypeId::of::<ModelMetadataProvider>())
    );
    assert_eq!(error.actual_adapter_type(), Some(TypeId::of::<u32>()));
    assert_eq!(error.sources(), std::slice::from_ref(&expected_source));
    assert_eq!(
        error.origins(),
        &[CapabilityOrigin::Registered {
            source: expected_source
        }]
    );
}

#[test]
fn test_model_registry_global_preserves_nested_reflection_error_chain() {
    let error = ModelRegistry::try_global().expect_err("global capability conflict must fail");
    assert_eq!(error.kind(), ModelRegistryErrorKind::ReflectionRegistry);
    let reflection = std::error::Error::source(&error)
        .and_then(|cause| cause.downcast_ref::<RegistryError>())
        .expect("model error must expose registry error");
    assert_eq!(
        reflection.capability_target(),
        Some(CapabilityTarget::Type(TypeId::of::<GlobalConflict>()))
    );
    assert_eq!(
        reflection.capability_id().expect("capability ID").as_str(),
        "model.test.global_context"
    );
    let (left, right) = reflection
        .conflicting_fragments()
        .expect("both conflicting capability sources");
    assert_ne!(left, right);
    assert!(!left.declaring_crate().is_empty());
    assert!(!right.declaring_crate().is_empty());
    assert_eq!(left.member_kind(), "capability");
    assert_eq!(right.member_kind(), "capability");
    let details = reflection.capability_details().expect("conflict details");
    assert_eq!(details.kind(), CapabilityConflictKind::AdapterTypeMismatch);
    let adapter_types = [details.first_adapter_type(), details.second_adapter_type()];
    assert!(adapter_types.contains(&TypeId::of::<u32>()));
    assert!(adapter_types.contains(&TypeId::of::<u64>()));
    assert_eq!(
        std::error::Error::source(reflection).and_then(|cause| cause.downcast_ref::<CapabilityConflict>()),
        Some(details)
    );
}

#[test]
fn test_public_builder_preserves_conflict_context_and_error_chain() {
    let target = TypeDescriptor::of::<DiagnosticsTarget>();
    let mut builder = RegistrySnapshotBuilder::new();
    builder.add_type_capabilities(
        target,
        vec![CapabilityDescriptor::with_adapter(key("model.test.context"), 7_u32)],
        source("context-left", 10),
    );
    builder.add_type_capabilities(
        target,
        vec![CapabilityDescriptor::with_adapter(key("model.test.context"), 9_u64)],
        source("context-right", 20),
    );

    let error = builder.build().expect_err("adapter mismatch must be rejected");

    assert_eq!(error.kind(), RegistryErrorKind::CapabilityConflict);
    assert_eq!(
        error.capability_target(),
        Some(CapabilityTarget::Type(TypeId::of::<DiagnosticsTarget>()))
    );
    assert_eq!(
        error.capability_id().expect("capability ID").as_str(),
        "model.test.context"
    );
    let detail = error.capability_details().expect("conflict details");
    assert_eq!(detail.kind(), CapabilityConflictKind::AdapterTypeMismatch);
    assert_eq!(detail.first_adapter_type(), TypeId::of::<u32>());
    assert_eq!(detail.second_adapter_type(), TypeId::of::<u64>());

    let (left, right) = error.conflicting_fragments().expect("source identities");
    assert_eq!(left, &source("context-left", 10));
    assert_eq!(right, &source("context-right", 20));
    assert_eq!(
        std::error::Error::source(&error).and_then(|cause| cause.downcast_ref::<CapabilityConflict>()),
        Some(detail)
    );
}

#[test]
fn test_public_builder_normalizes_source_order_for_duplicate_category() {
    let target = TypeDescriptor::of::<DiagnosticsTarget>();
    let mut forward = RegistrySnapshotBuilder::new();
    forward.add_type_capabilities(
        target,
        vec![CapabilityDescriptor::with_adapter(key("model.test.duplicate"), 1_u32)],
        source("duplicate-left", 30),
    );
    forward.add_type_capabilities(
        target,
        vec![CapabilityDescriptor::with_adapter(key("model.test.duplicate"), 2_u32)],
        source("duplicate-right", 40),
    );

    let mut reverse = RegistrySnapshotBuilder::new();
    reverse.add_type_capabilities(
        target,
        vec![CapabilityDescriptor::with_adapter(key("model.test.duplicate"), 2_u32)],
        source("duplicate-right", 40),
    );
    reverse.add_type_capabilities(
        target,
        vec![CapabilityDescriptor::with_adapter(key("model.test.duplicate"), 1_u32)],
        source("duplicate-left", 30),
    );

    let forward = forward.build().expect_err("duplicate ID must be rejected");
    let reverse = reverse.build().expect_err("duplicate ID must be rejected");
    assert_eq!(forward, reverse);
    assert_eq!(
        forward.capability_target(),
        Some(CapabilityTarget::Type(TypeId::of::<DiagnosticsTarget>()))
    );
    assert_eq!(
        forward.capability_details().expect("conflict details").kind(),
        CapabilityConflictKind::DuplicateId
    );
    assert_eq!(
        forward
            .capability_details()
            .expect("conflict details")
            .first_adapter_type(),
        TypeId::of::<u32>()
    );
    assert_eq!(
        forward
            .capability_details()
            .expect("conflict details")
            .second_adapter_type(),
        TypeId::of::<u32>()
    );
}
