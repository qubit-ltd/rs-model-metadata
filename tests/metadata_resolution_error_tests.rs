// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Metadata registration failures retain typed causes at vNext boundaries.

use std::sync::OnceLock;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_model_derive::Model;
use qubit_model_metadata::__private::ModelMetadataCapability;
use qubit_model_metadata::__private::ModelMetadataProvider;
use qubit_model_metadata::__private::v8;
use qubit_model_metadata::__private::v8::model_metadata_registration;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::registry::ModelRegistryErrorKind;
use qubit_model_metadata::registry::ModelRegistryOrigin;
use qubit_model_metadata::resolve::ResolveErrorKind;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectionContext;
use qubit_reflect::capability::CapabilityRegistration;
use qubit_reflect::error::ContextIssue;

#[Model(id = "error.DuplicateCapability")]
struct DuplicateCapabilityFixture {
    value: String,
}

#[Reflect]
struct WrongMetadataFixture<const N: usize> {
    value: String,
}

fn wrong_metadata() -> &'static TypeMetadata {
    static METADATA: OnceLock<TypeMetadata> = OnceLock::new();
    METADATA.get_or_init(|| {
        v8::GeneratedTypeMetadataBuilder::new::<WrongMetadataFixture<2>>(None, &[], v8::leak(v8::model_role()))
            .finish_unchecked()
    })
}

static PANIC_PROVIDER_CALLS: AtomicUsize = AtomicUsize::new(0);

#[Model(id = "error.PanicProvider")]
struct PanicProviderFixture {
    value: String,
}

fn panic_once_metadata() -> &'static TypeMetadata {
    if PANIC_PROVIDER_CALLS.fetch_add(1, Ordering::SeqCst) == 0 {
        panic!("intentional provider panic on the first projection");
    }
    TypeMetadata::of::<PanicProviderFixture>()
}

#[Model(id = "error.SourceMissingTarget")]
struct SourceMissingTargetFixture {
    #[reference(entity_id = "error.MissingTarget")]
    target: qubit_id::Id,
}

fn source() -> &'static ModelRegistryOrigin {
    static SOURCE: OnceLock<ModelRegistryOrigin> = OnceLock::new();
    SOURCE.get_or_init(|| ModelRegistryOrigin {
        crate_name: "metadata-resolution-errors",
        crate_version: None,
        module_path: module_path!(),
        file: Some(file!()),
        line: Some(line!()),
        column: Some(1),
    })
}

#[test]
fn test_duplicate_typed_metadata_capability_fails_context_construction() {
    let result = ReflectionContext::builder()
        .include_type::<DuplicateCapabilityFixture>()
        .register(model_metadata_registration::<DuplicateCapabilityFixture>())
        .register(model_metadata_registration::<DuplicateCapabilityFixture>())
        .build();
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("one target cannot receive duplicate typed capabilities"),
    };
    assert!(matches!(
        error.primary_issue(),
        ContextIssue::ConflictingCapability { .. }
    ));
    assert_eq!(error.issues().len(), 1);
}

#[test]
fn test_registry_projection_distinguishes_absence_from_metadata_abi_failure() {
    let context = ReflectionContext::builder()
        .include_type::<WrongMetadataFixture<1>>()
        .include_type::<String>()
        .register(CapabilityRegistration::for_type::<
            WrongMetadataFixture<1>,
            ModelMetadataCapability,
        >(wrong_metadata as ModelMetadataProvider))
        .build()
        .expect("typed capability registration is structurally valid");

    let error = ModelRegistry::from_context(&context)
        .expect_err("metadata for a different const instance must fail ABI validation");
    assert_eq!(error.kind(), ModelRegistryErrorKind::InvalidMetadata);
    assert!(error.abi_cause().is_some());
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn test_registry_caches_absent_metadata_in_a_valid_context() {
    let context = ReflectionContext::builder()
        .include_type::<String>()
        .include_type::<u8>()
        .build()
        .expect("reflection-only context");
    let registry = ModelRegistry::from_context(&context).expect("valid empty model projection");
    let missing = context
        .type_by_id(std::any::TypeId::of::<u8>())
        .expect("included descriptor");

    assert!(registry.metadata_for(&missing).expect("first lookup").is_none());
    assert!(registry.metadata_for(&missing).expect("cached lookup").is_none());
}

#[test]
fn test_panicking_metadata_provider_can_be_retried_during_registry_projection() {
    PANIC_PROVIDER_CALLS.store(0, Ordering::SeqCst);
    let context = ReflectionContext::builder()
        .include_type::<PanicProviderFixture>()
        .include_type::<String>()
        .register(CapabilityRegistration::for_type::<
            PanicProviderFixture,
            ModelMetadataCapability,
        >(panic_once_metadata))
        .build()
        .expect("typed metadata capability registration");

    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = ModelRegistry::from_context(&context);
    }));
    assert!(panic.is_err());
    let registry = ModelRegistry::from_context(&context).expect("retry after provider panic");
    assert!(std::ptr::eq(
        registry.metadata("error.PanicProvider").expect("registered model"),
        TypeMetadata::of::<PanicProviderFixture>()
    ));
    assert_eq!(PANIC_PROVIDER_CALLS.load(Ordering::SeqCst), 2);
}

#[test]
fn test_resolver_reports_missing_target_after_valid_metadata_projection() {
    let source_metadata = TypeMetadata::of::<SourceMissingTargetFixture>();
    let context = ReflectionContext::builder()
        .include_type::<SourceMissingTargetFixture>()
        .include_type::<qubit_id::Id>()
        .build()
        .expect("local context contains both reflected types without linked metadata");
    let registry = ModelRegistry::from_static_metadata(&context, &[(source_metadata, source())])
        .expect("source metadata matches its descriptor");
    let errors = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: &registry,
    })
    .resolve()
    .expect_err("missing targets must prevent graph publication");

    assert_eq!(errors.errors().len(), 1);
    assert_eq!(errors.errors()[0].kind(), ResolveErrorKind::MissingModelId);
    assert_eq!(errors.errors()[0].model_id(), Some("error.MissingTarget"));
}
