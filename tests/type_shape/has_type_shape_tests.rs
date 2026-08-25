// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::collections::HashMap;
use std::collections::HashSet;

use qubit_model_metadata::HasTypeShape;
use qubit_model_metadata::ScalarType;
use qubit_model_metadata::TypeCapabilities;
use qubit_model_metadata::TypeRef;
use qubit_model_metadata::TypeShape;

#[test]
fn test_has_type_shape_exposes_scalar_shape() {
    assert!(matches!(
        <i32 as HasTypeShape>::TYPE_SHAPE,
        TypeShape::Scalar(ScalarType::I32)
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
fn test_container_capabilities_describe_the_outer_layer() {
    assert_eq!(<Option<String> as HasTypeShape>::CAPABILITIES, TypeCapabilities::TEXT);
    assert_eq!(<Vec<String> as HasTypeShape>::CAPABILITIES, TypeCapabilities::SEQUENCE);
    assert_eq!(<HashSet<String> as HasTypeShape>::CAPABILITIES, TypeCapabilities::SET);
    assert_eq!(<HashSet<String> as HasTypeShape>::ELEMENT_CAPABILITIES, None);
    assert_eq!(
        <HashMap<String, Vec<String>> as HasTypeShape>::CAPABILITIES,
        TypeCapabilities::MAP
    );
}

#[cfg(feature = "chrono")]
#[test]
fn test_chrono_types_have_temporal_capability_and_scalar_shapes() {
    use chrono::DateTime;
    use chrono::NaiveDate;
    use chrono::NaiveDateTime;
    use chrono::NaiveTime;
    use chrono::Utc;

    assert_eq!(<NaiveDate as HasTypeShape>::CAPABILITIES, TypeCapabilities::TEMPORAL);
    assert!(matches!(
        TypeRef::of::<NaiveDate>().shape(),
        TypeShape::Scalar(ScalarType::Date)
    ));
    assert_eq!(<NaiveTime as HasTypeShape>::CAPABILITIES, TypeCapabilities::TEMPORAL);
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

    assert_eq!(<BigDecimal as HasTypeShape>::CAPABILITIES, TypeCapabilities::DECIMAL);
    assert!(matches!(
        TypeRef::of::<BigDecimal>().shape(),
        TypeShape::Scalar(ScalarType::BigDecimal)
    ));
}

#[cfg(feature = "id")]
#[test]
fn test_id_has_u64_scalar_shape() {
    use qubit_id::Id;

    assert_eq!(<Id as HasTypeShape>::CAPABILITIES, TypeCapabilities::NONE);
    assert!(matches!(
        TypeRef::of::<Id>().shape(),
        TypeShape::Scalar(ScalarType::U64)
    ));
}
