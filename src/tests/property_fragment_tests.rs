// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Crate-private property fragment behavior checks.

use std::any::TypeId;

use qubit_reflect::Reflect;
use qubit_reflect::ReflectionContext;
use qubit_reflect::descriptor::TypeDescriptor;

use crate::field_metadata::FieldMetadata;
use crate::metadata::PropertyFragmentSource;
use crate::property_fragment::PropertyFragment;

#[Reflect]
struct FragmentCoverageFixture {
    name: String,
}

#[test]
fn test_debug_and_unbound_type_expression_fallback_are_observable() {
    let context = ReflectionContext::global();
    let descriptor = TypeDescriptor::of_in::<FragmentCoverageFixture>(context)
        .field("name")
        .expect("fixture field");
    let field = Box::leak(Box::new(FieldMetadata::from_reflect(
        TypeId::of::<FragmentCoverageFixture>(),
        descriptor,
    )));
    let fragment = PropertyFragment::new("name", None, PropertyFragmentSource::Field(field));
    let empty_context = ReflectionContext::builder().build().expect("empty reflection context");

    assert!(format!("{fragment:?}").contains("PropertyFragment"));
    assert!(fragment.type_expression_in(&empty_context).is_none());
}
