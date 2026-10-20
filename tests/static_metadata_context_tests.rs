// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_derive::Model;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
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
fn static_metadata_can_be_reused_with_distinct_context_bound_descriptors() {
    let metadata = TypeMetadata::of::<ContextBoundStaticFacts>();
    let first_context = context();
    let second_context = context();

    let first = metadata.descriptor_in(&first_context);
    let second = metadata.descriptor_in(&second_context);

    assert_eq!(first.type_id(), second.type_id());
    assert!(!std::ptr::eq(first.context(), second.context()));
}

#[test]
fn registry_queries_reuse_static_metadata_without_crossing_contexts() {
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
fn opaque_field_keeps_static_facts_and_stays_hidden_from_reflect_descriptor() {
    let metadata = TypeMetadata::of::<ContextBoundStaticFacts>();
    let context = context();
    let field = metadata.field("opaque").expect("opaque field metadata");

    assert!(field.is_opaque(), "the M static facts preserve the traversal boundary");
    assert!(
        TypeDescriptor::of_in::<ContextBoundStaticFacts>(&context)
            .fields()
            .get("opaque")
            .is_none(),
        "R omits a typed field descriptor when the field is opaque",
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
