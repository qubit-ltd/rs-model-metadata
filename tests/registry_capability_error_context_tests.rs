// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Verifies context-bound registry capability lookup and error provenance.

use qubit_model_derive::Model;
use qubit_model_metadata::__private::v8::model_metadata_registration;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::registry::ModelRegistryErrorKind;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectionContext;
use qubit_reflect::capability::CapabilityError;
use qubit_reflect::capability::CapabilityRegistration;
use qubit_reflect::capability::CapabilitySpec;

#[Model(id = "example.RegistryContextTarget")]
struct RegistryContextTarget;

#[Reflect]
struct WrongAdapterTarget;

struct WrongMetadataAdapter;

impl CapabilitySpec for WrongMetadataAdapter {
    type Adapter = u32;

    const ID: &'static str = "qubit.model.metadata.v1";
}

#[test]
fn test_registry_capability_error_retains_typed_cause() {
    let context = ReflectionContext::builder()
        .include_type::<WrongAdapterTarget>()
        .register(CapabilityRegistration::for_type::<
            WrongAdapterTarget,
            WrongMetadataAdapter,
        >(7_u32))
        .build()
        .expect("valid context");

    let error = ModelRegistry::from_context(&context).expect_err("wrong model metadata adapter must fail");

    assert_eq!(error.kind(), ModelRegistryErrorKind::Capability);
    assert!(
        std::error::Error::source(&error)
            .and_then(|cause| cause.downcast_ref::<CapabilityError>())
            .is_some()
    );
}

#[test]
fn test_registry_uses_only_its_bound_context_and_rebinds_descriptors() {
    let first_context = ReflectionContext::builder()
        .include_type::<RegistryContextTarget>()
        .register(model_metadata_registration::<RegistryContextTarget>())
        .build()
        .expect("registered model context");
    let second_context = ReflectionContext::builder()
        .include_type::<RegistryContextTarget>()
        .build()
        .expect("unregistered model context");
    let registry = ModelRegistry::from_context(&first_context).expect("registry projection");
    let first_descriptor = first_context
        .type_by_id(std::any::TypeId::of::<RegistryContextTarget>())
        .expect("first descriptor");
    let second_descriptor = second_context
        .type_by_id(std::any::TypeId::of::<RegistryContextTarget>())
        .expect("second descriptor");

    assert!(
        registry
            .metadata_for(&first_descriptor)
            .expect("first lookup")
            .is_some()
    );
    assert!(
        registry
            .metadata_for(&second_descriptor)
            .expect("lookup is rebound into registry context")
            .is_some()
    );
}
