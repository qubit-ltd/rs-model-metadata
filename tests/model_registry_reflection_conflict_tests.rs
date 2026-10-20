// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Verifies model registry failures retain context-bound capability causes.

use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::registry::ModelRegistryErrorKind;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectionContext;
use qubit_reflect::capability::CapabilityError;
use qubit_reflect::capability::CapabilityRegistration;
use qubit_reflect::capability::CapabilitySpec;

#[Reflect]
struct ConflictingMetadataCapability;

struct WrongModelMetadataCapability;

impl CapabilitySpec for WrongModelMetadataCapability {
    type Adapter = u32;

    const ID: &'static str = "qubit.model.metadata.v1";
}

#[test]
fn test_registry_error_preserves_context_capability_mismatch() {
    let context = ReflectionContext::builder()
        .include_type::<ConflictingMetadataCapability>()
        .register(CapabilityRegistration::for_type::<
            ConflictingMetadataCapability,
            WrongModelMetadataCapability,
        >(7_u32))
        .build()
        .expect("a typed registration with a colliding ID is valid until queried");

    let error = ModelRegistry::from_context(&context).expect_err("adapter contract mismatch must be reported");

    assert_eq!(error.kind(), ModelRegistryErrorKind::Capability);
    assert!(error.sources().is_empty());
    assert!(
        std::error::Error::source(&error)
            .and_then(|cause| cause.downcast_ref::<CapabilityError>())
            .is_some()
    );
}
