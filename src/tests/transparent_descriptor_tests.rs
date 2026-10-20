// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Transparent traversal stops when a nested relation is absent from context.

use std::any::TypeId;

use qubit_reflect::ReflectionContext;

use crate::transparent_descriptor::transparent_descriptor;

#[test]
fn test_transparent_descriptor_traverses_optional_and_pointer_layers() {
    let context = ReflectionContext::builder()
        .include_type::<Option<Box<String>>>()
        .include_type::<Box<String>>()
        .include_type::<String>()
        .build()
        .expect("transparent wrapper types should register");

    let descriptor = transparent_descriptor(context.type_of::<Option<Box<String>>>())
        .expect("registered wrapper chain should resolve");
    assert_eq!(descriptor.type_id(), TypeId::of::<String>());
    assert!(std::ptr::eq(descriptor.context(), &context));
}

#[test]
fn test_transparent_descriptor_resolves_static_generic_arguments_in_context() {
    let context = ReflectionContext::builder()
        .include_type::<Option<Box<String>>>()
        .build()
        .expect("outer wrapper should register");

    let descriptor = transparent_descriptor(context.type_of::<Option<Box<String>>>())
        .expect("nested static generic arguments remain available");
    assert_eq!(descriptor.type_id(), TypeId::of::<String>());
    assert!(std::ptr::eq(descriptor.context(), &context));
}
