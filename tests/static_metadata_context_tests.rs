// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Verifies that static model metadata remains isolated across reflection
//! contexts.

use qubit_model_derive::Model;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;

#[derive(Clone, Debug, PartialEq)]
struct OpaqueFieldType;

#[Model(no_redact, no_serialize, no_deserialize)]
struct ContextBoundStaticFacts {
    name: String,
    #[opaque]
    opaque: OpaqueFieldType,
}

#[Model]
struct AnonymousNestedChild {
    name: String,
}

#[Model]
struct AnonymousNestedParent {
    child: AnonymousNestedChild,
}

#[cfg(feature = "generic")]
#[Model]
struct GenericAnonymousNestedChild<T> {
    name: T,
}

#[cfg(feature = "generic")]
#[Model]
struct GenericAnonymousNestedParent {
    child: GenericAnonymousNestedChild<String>,
}

/// Builds a reflection context containing the fixture's registration and field
/// types.
fn context() -> ReflectionContext {
    ReflectionContext::builder()
        .include_type::<ContextBoundStaticFacts>()
        .include_type::<String>()
        .register(qubit_model_metadata::__private::v8::model_metadata_registration::<
            ContextBoundStaticFacts,
        >())
        .build()
        .expect("reflection context should build")
}

#[test]
fn test_static_metadata_can_be_reused_with_distinct_context_bound_descriptors() {
    let metadata = TypeMetadata::of::<ContextBoundStaticFacts>();
    let first_context = context();
    let second_context = context();

    let first = metadata.descriptor_in(&first_context);
    let second = metadata.descriptor_in(&second_context);

    assert_eq!(first.type_id(), second.type_id());
    assert!(!std::ptr::eq(first.context(), second.context()));
}

#[test]
fn test_registry_queries_reuse_static_metadata_without_crossing_contexts() {
    let metadata = TypeMetadata::of::<ContextBoundStaticFacts>();
    let first_context = context();
    let second_context = context();
    let first_descriptor = metadata.descriptor_in(&first_context);
    let second_descriptor = metadata.descriptor_in(&second_context);
    let first_registry = ModelRegistry::from_context(&first_context).expect("first model registry");
    let second_registry = ModelRegistry::from_context(&second_context).expect("second model registry");

    let first_metadata = first_registry
        .metadata_for(&first_descriptor)
        .expect("first context capability query")
        .expect("first context model capability");
    let second_metadata = second_registry
        .metadata_for(&second_descriptor)
        .expect("second context capability query")
        .expect("second context model capability");

    assert!(std::ptr::eq(first_metadata, metadata));
    assert!(std::ptr::eq(second_metadata, metadata));
    assert!(std::ptr::eq(first_descriptor.context(), &first_context));
    assert!(std::ptr::eq(second_descriptor.context(), &second_context));
    assert!(!std::ptr::eq(first_descriptor.context(), second_descriptor.context()));
}

#[test]
fn test_opaque_field_keeps_static_facts_and_stays_hidden_from_reflect_descriptor() {
    let metadata = TypeMetadata::of::<ContextBoundStaticFacts>();
    let context = context();
    let field = metadata.field("opaque").expect("opaque field metadata");

    assert!(
        field.is_opaque(),
        "static metadata preserves the opaque field traversal boundary"
    );
    assert!(
        TypeDescriptor::of_in::<ContextBoundStaticFacts>(&context)
            .fields()
            .get("opaque")
            .is_none(),
        "reflection omits a typed field descriptor when the field is opaque",
    );
    assert_eq!(
        field
            .type_expression_in(&context)
            .expect("field type expression")
            .display()
            .to_string(),
        "OpaqueFieldType",
    );
    ModelRegistry::from_context(&context).expect("opaque static field facts match the skipped reflection field");
}

fn nested_context(include_child: bool, register_child: bool) -> ReflectionContext {
    let builder = ReflectionContext::builder()
        .include_type::<AnonymousNestedParent>()
        .include_type::<String>();
    let builder = if include_child {
        builder.include_type::<AnonymousNestedChild>()
    } else {
        builder
    };
    let builder = if register_child {
        builder.register(qubit_model_metadata::__private::v8::model_metadata_registration::<
            AnonymousNestedChild,
        >())
    } else {
        builder
    };
    builder.build().expect("nested reflection context should build")
}

#[test]
fn test_static_registry_does_not_infer_child_metadata_from_reflection_type() {
    let reflection = nested_context(true, false);
    let registry = ModelRegistry::from_static_metadata(&reflection, &[]).expect("empty static registry");
    let root = TypeMetadata::of::<AnonymousNestedParent>();
    let child = TypeMetadata::of::<AnonymousNestedChild>();

    let graph = StructureResolver::for_static_roots(&registry, [root])
        .resolve()
        .expect("unregistered child metadata is not a resolution error");

    assert!(registry.entries().is_empty());
    assert!(graph.model(root.type_id()).is_some());
    assert!(
        graph.model(child.type_id()).is_none(),
        "reflection type inclusion alone does not provide model metadata"
    );
}

#[test]
fn test_static_registry_uses_registered_context_child_provider() {
    let reflection = nested_context(true, true);
    let registry = ModelRegistry::from_static_metadata(&reflection, &[]).expect("empty static registry");
    let root = TypeMetadata::of::<AnonymousNestedParent>();
    let child = TypeMetadata::of::<AnonymousNestedChild>();

    let graph = StructureResolver::for_static_roots(&registry, [root])
        .resolve()
        .expect("registered context provider supplies child metadata");

    assert!(
        registry.entries().is_empty(),
        "context capabilities are not static index entries"
    );
    assert!(graph.model(child.type_id()).is_some());
}

#[test]
fn test_static_registry_leaves_child_absent_when_context_lacks_child_type() {
    let reflection = nested_context(false, false);
    let registry = ModelRegistry::from_static_metadata(&reflection, &[]).expect("empty static registry");
    let root = TypeMetadata::of::<AnonymousNestedParent>();
    let child = TypeMetadata::of::<AnonymousNestedChild>();

    let graph = StructureResolver::for_static_roots(&registry, [root])
        .resolve()
        .expect("an unprovided child descriptor leaves the root graph valid");

    assert!(registry.entries().is_empty());
    assert!(graph.model(root.type_id()).is_some());
    assert!(graph.model(child.type_id()).is_none());
}

#[cfg(feature = "generic")]
#[test]
fn test_generated_nested_provider_discovers_concrete_generic_child() {
    let reflection = ReflectionContext::builder()
        .include_type::<GenericAnonymousNestedParent>()
        .include_type::<GenericAnonymousNestedChild<String>>()
        .include_type::<String>()
        .build()
        .expect("generic nested reflection context should build");
    let registry = ModelRegistry::from_static_metadata(&reflection, &[]).expect("empty static registry");
    let root = TypeMetadata::of::<GenericAnonymousNestedParent>();
    let child = TypeMetadata::of::<GenericAnonymousNestedChild<String>>();

    let graph = StructureResolver::for_static_roots(&registry, [root])
        .resolve()
        .expect("generated nested provider discovers the concrete generic child");

    assert!(registry.entries().is_empty());
    assert!(graph.model(root.type_id()).is_some());
    assert!(graph.model(child.type_id()).is_some());
}
