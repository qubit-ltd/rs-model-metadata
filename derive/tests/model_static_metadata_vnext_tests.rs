// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use model_runtime::metadata::TypeMetadata;
use qubit_model_derive::Model;
use qubit_reflect::ReflectionContext;

#[derive(Clone, Debug, PartialEq)]
struct UnregisteredField;

#[Model(no_redact, no_serialize, no_deserialize)]
struct ExternalConsumerModel {
    label: String,
    #[opaque]
    unregistered: UnregisteredField,
}

#[Model(no_redact, no_serialize, no_deserialize)]
struct BorrowedStrModel {
    value: &'static str,
}

fn context() -> ReflectionContext {
    ReflectionContext::builder()
        .include_type::<ExternalConsumerModel>()
        .include_type::<String>()
        .build()
        .expect("reflection context builds")
}

#[test]
fn test_generated_static_metadata_binds_descriptors_to_each_context() {
    let metadata = TypeMetadata::of::<ExternalConsumerModel>();
    let first = context();
    let second = context();
    let first_type = metadata.descriptor_in(&first);
    let second_type = metadata.descriptor_in(&second);

    assert_eq!(first_type.type_id(), second_type.type_id());
    assert!(!std::ptr::eq(first_type.context(), second_type.context()));

    let field = metadata.field("unregistered").expect("field metadata");
    assert!(field.reflect_in(&first).is_none());
    assert!(field.reflect_in(&second).is_none());
    let first_expression = field
        .type_expression_in(&first)
        .expect("field type expression in first context");
    let second_expression = field
        .type_expression_in(&second)
        .expect("field type expression in second context");
    assert!(!std::ptr::eq(first_expression.context(), second_expression.context(),));
    assert_eq!(first_expression.display().to_string(), "UnregisteredField",);
    assert!(
        first
            .find_type_by_id(std::any::TypeId::of::<UnregisteredField>())
            .expect("context lookup")
            .is_none()
    );
}

#[test]
fn test_generated_static_metadata_accepts_borrowed_str_fields() {
    let metadata = TypeMetadata::of::<BorrowedStrModel>();
    assert_eq!(metadata.field("value").expect("field metadata").name(), Some("value"));
}
