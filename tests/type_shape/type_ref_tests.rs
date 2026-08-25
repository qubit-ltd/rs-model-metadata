// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::collections::BTreeMap;
use std::collections::HashSet;

use qubit_model_metadata::ScalarType;
use qubit_model_metadata::TypeCapabilities;
use qubit_model_metadata::TypeRef;
use qubit_model_metadata::TypeShape;

struct OpaqueType;
struct OtherOpaqueType;

#[test]
fn test_type_ref_retains_the_rust_type_name() {
    assert!(TypeRef::of::<i32>().type_name().ends_with("i32"));
}

#[test]
fn test_opaque_type_ref_reports_opaque_shape_and_no_capabilities() {
    let type_ref = TypeRef::opaque::<OpaqueType>();

    assert!(matches!(type_ref.shape(), TypeShape::Opaque));
    assert_eq!(type_ref.capabilities(), TypeCapabilities::NONE);
    assert_eq!(type_ref.element_capabilities(), None);
    assert_eq!(type_ref.strip_optional().type_name(), type_ref.type_name());
    assert!(type_ref.named_metadata().is_none());
}

/// Verifies that opaque references retain an identity usable for structural
/// relation validation without comparing Rust type-name text.
#[test]
fn test_opaque_type_ref_retains_type_identity() {
    assert_eq!(
        TypeRef::opaque::<OpaqueType>().identity(),
        TypeRef::opaque::<OpaqueType>().identity()
    );
    assert_ne!(
        TypeRef::opaque::<OpaqueType>().identity(),
        TypeRef::opaque::<OtherOpaqueType>().identity()
    );
}

#[test]
fn test_type_ref_of_nested_option_vector_preserves_each_layer() {
    let shape = TypeRef::of::<Option<Vec<String>>>().shape();
    assert!(matches!(shape, TypeShape::Optional(inner)
        if matches!(inner.shape(), TypeShape::Sequence(element)
            if matches!(element.shape(), TypeShape::Scalar(ScalarType::String)))));
}

#[test]
fn test_type_ref_of_array_exposes_const_length() {
    assert!(matches!(
        TypeRef::of::<[u16; 3]>().shape(),
        TypeShape::Array { length: 3, .. }
    ));
}

#[test]
fn test_type_ref_exposes_rust_type_name() {
    let type_ref = TypeRef::of::<Option<Vec<String>>>();

    assert_eq!(type_ref.type_name(), core::any::type_name::<Option<Vec<String>>>());
}

#[test]
fn test_type_ref_strip_optional_removes_only_the_outer_layer() {
    let stripped = TypeRef::of::<Option<Vec<String>>>().strip_optional();

    assert!(matches!(stripped.shape(), TypeShape::Sequence(element)
        if matches!(element.shape(), TypeShape::Scalar(ScalarType::String))));
}

#[test]
fn test_set_and_map_shapes_recurse_into_unordered_primitive_types() {
    assert!(matches!(TypeRef::of::<HashSet<f32>>().shape(), TypeShape::Set(element)
        if matches!(element.shape(), TypeShape::Scalar(ScalarType::F32))));
    assert!(
        matches!(TypeRef::of::<BTreeMap<f32, String>>().shape(), TypeShape::Map { key, value }
        if matches!(key.shape(), TypeShape::Scalar(ScalarType::F32))
            && matches!(value.shape(), TypeShape::Scalar(ScalarType::String)))
    );
}
