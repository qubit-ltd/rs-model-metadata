// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Context-bound metadata lookup contracts for the public model registry API.

use std::any::TypeId;
use std::sync::OnceLock;

use qubit_model_metadata::__private::ModelMetadataCapability;
use qubit_model_metadata::__private::ModelMetadataProvider;
use qubit_model_metadata::__private::v8;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::capability::CapabilityRegistration;

#[Reflect]
struct AnonymousMetadataFixture;

fn first_context_metadata() -> &'static TypeMetadata {
    static METADATA: OnceLock<&'static TypeMetadata> = OnceLock::new();
    METADATA.get_or_init(anonymous_metadata)
}

fn second_context_metadata() -> &'static TypeMetadata {
    static METADATA: OnceLock<&'static TypeMetadata> = OnceLock::new();
    METADATA.get_or_init(anonymous_metadata)
}

fn anonymous_metadata() -> &'static TypeMetadata {
    v8::leak(
        v8::GeneratedTypeMetadataBuilder::new::<AnonymousMetadataFixture>(None, &[], v8::leak(v8::model_role()))
            .finish::<AnonymousMetadataFixture>(),
    )
}

#[test]
fn test_metadata_cache_is_scoped_to_its_reflection_context() {
    let first_context = ReflectionContext::builder()
        .include_type::<AnonymousMetadataFixture>()
        .register(CapabilityRegistration::for_type::<
            AnonymousMetadataFixture,
            ModelMetadataCapability,
        >(first_context_metadata as ModelMetadataProvider))
        .build()
        .expect("first context");
    let second_context = ReflectionContext::builder()
        .include_type::<AnonymousMetadataFixture>()
        .register(CapabilityRegistration::for_type::<
            AnonymousMetadataFixture,
            ModelMetadataCapability,
        >(second_context_metadata as ModelMetadataProvider))
        .build()
        .expect("second context");
    assert_ne!(first_context.id(), second_context.id());

    let first_metadata = first_context_metadata();
    let second_metadata = second_context_metadata();
    assert!(!std::ptr::eq(first_metadata, second_metadata));
    let first = ModelRegistry::from_context(&first_context).expect("first context registry");
    let second = ModelRegistry::from_context(&second_context).expect("second context registry");
    let first_descriptor = first_context
        .type_by_id(TypeId::of::<AnonymousMetadataFixture>())
        .expect("registered type");
    let second_descriptor = second_context
        .type_by_id(TypeId::of::<AnonymousMetadataFixture>())
        .expect("registered type");

    assert!(std::ptr::eq(
        first.metadata_for(&first_descriptor).expect("first lookup").unwrap(),
        first_metadata,
    ));
    assert!(std::ptr::eq(
        second.metadata_for(&second_descriptor).expect("second lookup").unwrap(),
        second_metadata,
    ));
    assert!(std::ptr::eq(
        first
            .metadata_for(&second_descriptor)
            .expect("registry rebinds into its context")
            .unwrap(),
        first_metadata,
    ));
}

#[test]
fn test_metadata_lookup_returns_none_for_type_outside_registry_context() {
    let registry_context = ReflectionContext::builder()
        .include_type::<AnonymousMetadataFixture>()
        .build()
        .expect("registry context");
    let external_context = ReflectionContext::builder()
        .include_type::<String>()
        .build()
        .expect("external context");
    let registry = ModelRegistry::from_context(&registry_context).expect("registry");
    let external_descriptor = external_context
        .type_by_id(TypeId::of::<String>())
        .expect("string type");

    assert!(
        registry
            .metadata_for(&external_descriptor)
            .expect("a type absent from the registry context is not an error")
            .is_none()
    );
}

#[test]
fn test_empty_metadata_registry_exposes_empty_indexes() {
    let empty_context = ReflectionContext::builder().build().expect("empty context");
    let registry = ModelRegistry::from_static_metadata(&empty_context, &[]).expect("empty registry is valid");

    assert!(registry.entries().is_empty());
    assert!(registry.get("missing.Model").is_none());
    assert!(registry.source("missing.Model").is_none());
    assert!(registry.by_type_id(TypeId::of::<String>()).is_none());
    assert!(
        registry
            .metadata_for(&TypeDescriptor::of::<String>())
            .expect("metadata-only lookup is infallible")
            .is_none()
    );
    #[cfg(feature = "generic")]
    {
        assert!(registry.generic_definitions().is_empty());
    }
}
