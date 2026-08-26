// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Contracts and implementations for statically describable Rust types.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::collections::VecDeque;

#[cfg(feature = "big-decimal")]
use bigdecimal::BigDecimal;
#[cfg(feature = "chrono")]
use chrono::DateTime;
#[cfg(feature = "chrono")]
use chrono::NaiveDate;
#[cfg(feature = "chrono")]
use chrono::NaiveDateTime;
#[cfg(feature = "chrono")]
use chrono::NaiveTime;
#[cfg(feature = "chrono")]
use chrono::Utc;
#[cfg(feature = "id")]
use qubit_id::Id;

use crate::type_shape::ScalarType;
use crate::type_shape::TypeCapabilities;
use crate::type_shape::TypeRef;
use crate::type_shape::TypeShape;

/// Metadata exposed by types whose structure can be described statically.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::HasTypeShape;
/// use qubit_model_metadata::TypeCapabilities;
/// use qubit_model_metadata::TypeShape;
///
/// assert!(matches!(String::TYPE_SHAPE, TypeShape::Scalar(_)));
/// assert_eq!(String::CAPABILITIES, TypeCapabilities::TEXT);
/// ```
pub trait HasTypeShape: 'static {
    /// The recursive shape of this type.
    const TYPE_SHAPE: TypeShape;

    /// Capabilities supported by this type's outermost layer.
    const CAPABILITIES: TypeCapabilities;

    /// Capabilities of sequence elements, when the outer shape has elements
    /// that may carry constraints.
    const ELEMENT_CAPABILITIES: Option<TypeCapabilities> = None;
}

/// Implements [`HasTypeShape`] for scalar types that have no capabilities.
macro_rules! impl_scalar_type_shape {
    ($($type:ty => $scalar:ident),+ $(,)?) => {
        $(
            impl HasTypeShape for $type {
                const TYPE_SHAPE: TypeShape = TypeShape::Scalar(ScalarType::$scalar);
                const CAPABILITIES: TypeCapabilities = TypeCapabilities::NONE;
            }
        )+
    };
}

impl_scalar_type_shape! {
    bool => Bool,
    char => Char,
    i8 => I8,
    i16 => I16,
    i32 => I32,
    i64 => I64,
    i128 => I128,
    isize => Isize,
    u8 => U8,
    u16 => U16,
    u32 => U32,
    u64 => U64,
    u128 => U128,
    usize => Usize,
    f32 => F32,
    f64 => F64,
}

impl HasTypeShape for String {
    const TYPE_SHAPE: TypeShape = TypeShape::Scalar(ScalarType::String);
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::TEXT;
}

#[cfg(feature = "chrono")]
impl HasTypeShape for NaiveDate {
    const TYPE_SHAPE: TypeShape = TypeShape::Scalar(ScalarType::Date);
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::TEMPORAL;
}

#[cfg(feature = "chrono")]
impl HasTypeShape for NaiveTime {
    const TYPE_SHAPE: TypeShape = TypeShape::Scalar(ScalarType::Time);
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::TEMPORAL;
}

#[cfg(feature = "chrono")]
impl HasTypeShape for NaiveDateTime {
    const TYPE_SHAPE: TypeShape = TypeShape::Scalar(ScalarType::DateTime);
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::TEMPORAL;
}

#[cfg(feature = "chrono")]
impl HasTypeShape for DateTime<Utc> {
    const TYPE_SHAPE: TypeShape = TypeShape::Scalar(ScalarType::Instant);
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::TEMPORAL;
}

#[cfg(feature = "big-decimal")]
impl HasTypeShape for BigDecimal {
    const TYPE_SHAPE: TypeShape = TypeShape::Scalar(ScalarType::BigDecimal);
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::DECIMAL;
}

#[cfg(feature = "id")]
impl HasTypeShape for Id {
    const TYPE_SHAPE: TypeShape = TypeShape::Scalar(ScalarType::U64);
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::NONE;
}

impl<T: HasTypeShape> HasTypeShape for Option<T> {
    const TYPE_SHAPE: TypeShape = TypeShape::Optional(TypeRef::of::<T>());
    const CAPABILITIES: TypeCapabilities = T::CAPABILITIES;
    const ELEMENT_CAPABILITIES: Option<TypeCapabilities> = T::ELEMENT_CAPABILITIES;
}

impl<T: HasTypeShape> HasTypeShape for Vec<T> {
    const TYPE_SHAPE: TypeShape = TypeShape::Sequence(TypeRef::of::<T>());
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::SEQUENCE;
    const ELEMENT_CAPABILITIES: Option<TypeCapabilities> = Some(T::CAPABILITIES);
}

impl<T: HasTypeShape> HasTypeShape for LinkedList<T> {
    const TYPE_SHAPE: TypeShape = TypeShape::Sequence(TypeRef::of::<T>());
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::SEQUENCE;
    const ELEMENT_CAPABILITIES: Option<TypeCapabilities> = Some(T::CAPABILITIES);
}

impl<T: HasTypeShape> HasTypeShape for VecDeque<T> {
    const TYPE_SHAPE: TypeShape = TypeShape::Sequence(TypeRef::of::<T>());
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::SEQUENCE;
    const ELEMENT_CAPABILITIES: Option<TypeCapabilities> = Some(T::CAPABILITIES);
}

impl<T: HasTypeShape> HasTypeShape for BinaryHeap<T> {
    const TYPE_SHAPE: TypeShape = TypeShape::Sequence(TypeRef::of::<T>());
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::SEQUENCE;
    const ELEMENT_CAPABILITIES: Option<TypeCapabilities> = Some(T::CAPABILITIES);
}

impl<T: HasTypeShape> HasTypeShape for HashSet<T> {
    const TYPE_SHAPE: TypeShape = TypeShape::Set(TypeRef::of::<T>());
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::SET;
}

impl<T: HasTypeShape> HasTypeShape for BTreeSet<T> {
    const TYPE_SHAPE: TypeShape = TypeShape::Set(TypeRef::of::<T>());
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::SET;
}

impl<K: HasTypeShape, V: HasTypeShape> HasTypeShape for HashMap<K, V> {
    const TYPE_SHAPE: TypeShape = TypeShape::Map {
        key: TypeRef::of::<K>(),
        value: TypeRef::of::<V>(),
    };
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::MAP;
}

impl<K: HasTypeShape, V: HasTypeShape> HasTypeShape for BTreeMap<K, V> {
    const TYPE_SHAPE: TypeShape = TypeShape::Map {
        key: TypeRef::of::<K>(),
        value: TypeRef::of::<V>(),
    };
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::MAP;
}

impl<T: HasTypeShape, const N: usize> HasTypeShape for [T; N] {
    const TYPE_SHAPE: TypeShape = TypeShape::Array {
        element: TypeRef::of::<T>(),
        length: N,
    };
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::SEQUENCE.union(TypeCapabilities::ARRAY);
    const ELEMENT_CAPABILITIES: Option<TypeCapabilities> = Some(T::CAPABILITIES);
}
