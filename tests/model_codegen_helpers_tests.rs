// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for the model-owned reflection codegen helpers.

#![cfg(feature = "generic")]

use std::sync::OnceLock;

use qubit_model_metadata::__private::v8;
use qubit_model_metadata::__private::v8::GenericModelMetadataCapability;
use qubit_model_metadata::__private::v8::register_generic_model_capability;
use qubit_model_metadata::generic::GenericModelMetadata;
use qubit_model_metadata::metadata::ModelId;
use qubit_model_metadata::metadata::ModelRole;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::Reflect as ReflectType;
use qubit_reflect::descriptor::TypeDefinitionDescriptor;

#[Reflect]
#[allow(dead_code, reason = "the reflection derive registers this generic definition")]
struct FirstGeneric<T> {
    value: T,
}

#[Reflect]
#[allow(dead_code, reason = "the reflection derive registers this generic definition")]
enum SecondGeneric<T> {
    Value(T),
}

fn first_definition_path() -> &'static str {
    concat!(env!("CARGO_CRATE_NAME"), "::FirstGeneric")
}

fn second_definition_path() -> &'static str {
    concat!(env!("CARGO_CRATE_NAME"), "::SecondGeneric")
}

fn first_definition() -> TypeDefinitionDescriptor<'static> {
    ReflectionContext::global()
        .type_definition_by_name(first_definition_path())
        .expect("first generic definition")
}

fn second_definition() -> TypeDefinitionDescriptor<'static> {
    ReflectionContext::global()
        .type_definition_by_name(second_definition_path())
        .expect("second generic definition")
}

/// Returns the metadata attached to the first generic definition.
fn first_metadata() -> &'static GenericModelMetadata {
    static METADATA: OnceLock<GenericModelMetadata> = OnceLock::new();
    METADATA.get_or_init(|| {
        v8::generic_model_metadata(
            ModelId::new("example.FirstGeneric"),
            ModelRole::Model,
            first_definition_path(),
            &[],
            &[],
        )
    })
}

/// Returns the metadata attached to the second generic definition.
fn second_metadata() -> &'static GenericModelMetadata {
    static METADATA: OnceLock<GenericModelMetadata> = OnceLock::new();
    METADATA.get_or_init(|| {
        v8::generic_model_metadata(
            ModelId::new("example.SecondGeneric"),
            ModelRole::Enum,
            second_definition_path(),
            &[],
            &[],
        )
    })
}

mod registrations {
    use super::register_generic_model_capability;

    register_generic_model_capability! {
        definition_path = concat!(env!("CARGO_CRATE_NAME"), "::FirstGeneric"),
        capability = __qubit_reflect_v4_definition_capability_FirstGeneric,
        metadata = super::first_metadata,
        source = (env!("CARGO_PKG_NAME"), module_path!(), line!(), column!(), 0x1111),
    }

    register_generic_model_capability! {
        definition_path = concat!(env!("CARGO_CRATE_NAME"), "::SecondGeneric"),
        capability = __qubit_reflect_v4_definition_capability_SecondGeneric,
        metadata = super::second_metadata,
        source = (env!("CARGO_PKG_NAME"), module_path!(), line!(), column!(), 0x2222),
    }
}

/// Asserts that a context-bound descriptor preserves exact sized and unsized
/// Rust type identity.
fn assert_reflected_root<T: ReflectType + ?Sized>() {
    let descriptor = TypeDescriptor::of_in::<T>(ReflectionContext::global());
    assert_eq!(descriptor.type_id(), std::any::TypeId::of::<T>());
    assert_eq!(descriptor.rust_type_name(), std::any::type_name::<T>());
}

#[test]
fn test_context_bound_descriptors_preserve_sized_and_unsized_type_identity() {
    assert_reflected_root::<u32>();
    assert_reflected_root::<str>();
    assert_reflected_root::<[u32]>();
}

#[test]
fn test_generic_model_capabilities_preserve_providers_and_registration_sources() {
    let reflection = ReflectionContext::global();
    let models = ModelRegistry::from_context(reflection).expect("generic models must project");

    let cases = [
        (first_definition(), first_metadata(), "example.FirstGeneric"),
        (second_definition(), second_metadata(), "example.SecondGeneric"),
    ];
    for (definition, expected_metadata, model_id) in cases {
        let provider = definition
            .find_capability::<GenericModelMetadataCapability>()
            .expect("capability contract")
            .expect("the definition must carry a generic model provider");
        assert!(std::ptr::eq((provider.adapter())(), expected_metadata));
        assert!(std::ptr::eq(
            models
                .generic(model_id)
                .expect("the projected generic model must exist"),
            expected_metadata,
        ));

        let capability_source = models.source(model_id).expect("capability source");
        assert_eq!(capability_source.crate_name, "explicit");
        assert!(!capability_source.module_path.is_empty());
        let declaration_source = models
            .get(model_id)
            .expect("projected entry")
            .declaration_source()
            .expect("the generic definition declaration source is retained");
        assert_eq!(capability_source, declaration_source);
        assert!(capability_source.module_path.ends_with("::TypeDefinitionMarker"));
        assert_eq!(capability_source.file, None);
        assert_eq!(capability_source.line, None);
        assert_eq!(capability_source.column, None);
    }

    let first_source = models
        .source("example.FirstGeneric")
        .expect("first generic model must have a registration source");
    let second_source = models
        .source("example.SecondGeneric")
        .expect("second generic model must have a registration source");
    assert_ne!(first_source, second_source);
}
