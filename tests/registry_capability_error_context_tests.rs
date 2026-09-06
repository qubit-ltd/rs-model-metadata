// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Verifies that downstream users can inspect capability conflict context.

use std::any::TypeId;

use qubit_model_metadata::__private::register_type_capabilities;
use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::ModelRegistryErrorKind;
use qubit_model_metadata::Reflect;
use qubit_model_metadata::register_reflected_type;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::capability::CapabilityConflict;
use qubit_reflect::capability::CapabilityConflictKind;
use qubit_reflect::capability::CapabilityDescriptor;
use qubit_reflect::capability::CapabilityKey;
use qubit_reflect::error::RegistryError;
use qubit_reflect::error::RegistryErrorKind;
use qubit_reflect::identity::CapabilityId;
use qubit_reflect::identity::FragmentIdentity;
use qubit_reflect::registry::CapabilityTarget;
use qubit_reflect::registry::RegistrySnapshotBuilder;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct DiagnosticsTarget;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct GlobalConflict;

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
