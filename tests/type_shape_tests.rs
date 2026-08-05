// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for type-shape metadata.

use std::collections::{
    BTreeMap,
    HashMap,
    HashSet,
};

use qubit_model_metadata::{
    HasTypeShape,
    ScalarType,
    TypeCapabilities,
    TypeRef,
    TypeShape,
};

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
fn test_array_has_sequence_and_fixed_array_capabilities() {
    assert_eq!(
        <[String; 3] as HasTypeShape>::CAPABILITIES,
        TypeCapabilities::SEQUENCE | TypeCapabilities::ARRAY
    );
}

#[test]
fn test_type_ref_exposes_rust_type_name() {
    let type_ref = TypeRef::of::<Option<Vec<String>>>();

    assert_eq!(
        type_ref.type_name(),
        core::any::type_name::<Option<Vec<String>>>()
    );
}

#[test]
fn test_type_ref_strip_optional_removes_only_the_outer_layer() {
    let stripped = TypeRef::of::<Option<Vec<String>>>().strip_optional();

    assert!(matches!(stripped.shape(), TypeShape::Sequence(element)
        if matches!(element.shape(), TypeShape::Scalar(ScalarType::String))));
}

#[test]
fn test_container_capabilities_describe_the_outer_layer() {
    assert_eq!(
        <Option<String> as HasTypeShape>::CAPABILITIES,
        TypeCapabilities::TEXT
    );
    assert_eq!(
        <Vec<String> as HasTypeShape>::CAPABILITIES,
        TypeCapabilities::SEQUENCE
    );
    assert_eq!(
        <HashSet<String> as HasTypeShape>::CAPABILITIES,
        TypeCapabilities::SET
    );
    assert_eq!(
        <HashSet<String> as HasTypeShape>::ELEMENT_CAPABILITIES,
        None
    );
    assert_eq!(
        <HashMap<String, Vec<String>> as HasTypeShape>::CAPABILITIES,
        TypeCapabilities::MAP
    );
}

#[test]
fn test_set_and_map_shapes_recurse_into_unordered_primitive_types() {
    assert!(
        matches!(TypeRef::of::<HashSet<f32>>().shape(), TypeShape::Set(element)
        if matches!(element.shape(), TypeShape::Scalar(ScalarType::F32)))
    );
    assert!(
        matches!(TypeRef::of::<BTreeMap<f32, String>>().shape(), TypeShape::Map { key, value }
        if matches!(key.shape(), TypeShape::Scalar(ScalarType::F32))
            && matches!(value.shape(), TypeShape::Scalar(ScalarType::String)))
    );
}

#[cfg(feature = "chrono")]
#[test]
fn test_chrono_types_have_temporal_capability_and_scalar_shapes() {
    use chrono::{
        DateTime,
        NaiveDate,
        NaiveDateTime,
        NaiveTime,
        Utc,
    };

    assert_eq!(
        <NaiveDate as HasTypeShape>::CAPABILITIES,
        TypeCapabilities::TEMPORAL
    );
    assert!(matches!(
        TypeRef::of::<NaiveDate>().shape(),
        TypeShape::Scalar(ScalarType::Date)
    ));
    assert_eq!(
        <NaiveTime as HasTypeShape>::CAPABILITIES,
        TypeCapabilities::TEMPORAL
    );
    assert!(matches!(
        TypeRef::of::<NaiveTime>().shape(),
        TypeShape::Scalar(ScalarType::Time)
    ));
    assert_eq!(
        <NaiveDateTime as HasTypeShape>::CAPABILITIES,
        TypeCapabilities::TEMPORAL
    );
    assert!(matches!(
        TypeRef::of::<NaiveDateTime>().shape(),
        TypeShape::Scalar(ScalarType::DateTime)
    ));
    assert_eq!(
        <DateTime<Utc> as HasTypeShape>::CAPABILITIES,
        TypeCapabilities::TEMPORAL
    );
    assert!(matches!(
        TypeRef::of::<DateTime<Utc>>().shape(),
        TypeShape::Scalar(ScalarType::Instant)
    ));
}

#[cfg(feature = "big-decimal")]
#[test]
fn test_big_decimal_has_decimal_capability_and_scalar_shape() {
    use bigdecimal::BigDecimal;

    assert_eq!(
        <BigDecimal as HasTypeShape>::CAPABILITIES,
        TypeCapabilities::DECIMAL
    );
    assert!(matches!(
        TypeRef::of::<BigDecimal>().shape(),
        TypeShape::Scalar(ScalarType::BigDecimal)
    ));
}
