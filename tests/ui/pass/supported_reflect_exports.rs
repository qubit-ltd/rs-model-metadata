// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_reflect::context::ReflectionContext;
use qubit_reflect::context::ReflectionContextId;
use qubit_reflect::descriptor::ConstExpression;
use qubit_reflect::descriptor::FieldDescriptor;
use qubit_reflect::descriptor::Reflect as ReflectTrait;
use qubit_reflect::descriptor::TypeDefinitionDescriptor;
use qubit_reflect::descriptor::TypeDefinitionId;
use qubit_reflect::descriptor::TypeDescriptor;
use qubit_reflect::descriptor::TypeExpression;
use qubit_reflect::descriptor::VariantDescriptor;
use qubit_reflect::error::RegistrationOrigin;
use qubit_reflect::Reflect;

#[Reflect]
struct Generic<T> {
    value: T,
}

fn main() {
    fn assert_reflect<T: ReflectTrait>() {}

    assert_reflect::<Generic<u8>>();
    let context = ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("linked reflection context");
    let descriptor = TypeDescriptor::of_in::<Generic<u8>>(&context);
    let _: TypeDefinitionDescriptor<'_> = descriptor.definition().expect("definition");
    let _: Option<TypeDefinitionId> = None;
    let _: Option<FieldDescriptor<'_>> = None;
    let _: Option<VariantDescriptor<'_>> = None;
    let _: Option<TypeExpression> = None;
    let _: Option<ConstExpression> = None;
    let _: Option<RegistrationOrigin> = None;
    let _: Option<ReflectionContextId> = None;
}
