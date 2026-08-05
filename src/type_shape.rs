// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow multiple-public-types
//! Type-level structural metadata for model fields.

// qubit-style: allow multiple-public-types

use core::any::type_name;
use std::collections::{
    BTreeMap,
    BTreeSet,
    HashMap,
    HashSet,
};

use bitflags::bitflags;

use crate::type_metadata::{
    NamedTypeRef,
    TypeMetadata,
};

bitflags! {
    /// Capabilities that determine which metadata attributes a type can accept.
    #[must_use]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct TypeCapabilities: u8 {
        /// The type accepts no metadata constraints.
        const NONE = 0;
        /// The type accepts text constraints.
        const TEXT = 1 << 0;
        /// The type accepts sequence constraints.
        const SEQUENCE = 1 << 1;
        /// The type is a set.
        const SET = 1 << 2;
        /// The type accepts map constraints.
        const MAP = 1 << 3;
        /// The type accepts temporal constraints.
        const TEMPORAL = 1 << 4;
        /// The type accepts decimal constraints.
        const DECIMAL = 1 << 5;
        /// The type is a fixed-length array.
        const ARRAY = 1 << 6;
    }
}

/// A scalar type supported by the metadata system.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScalarType {
    /// The Rust `bool` type.
    Bool,
    /// The Rust `char` type.
    Char,
    /// The Rust `i8` type.
    I8,
    /// The Rust `i16` type.
    I16,
    /// The Rust `i32` type.
    I32,
    /// The Rust `i64` type.
    I64,
    /// The Rust `i128` type.
    I128,
    /// The Rust `isize` type.
    Isize,
    /// The Rust `u8` type.
    U8,
    /// The Rust `u16` type.
    U16,
    /// The Rust `u32` type.
    U32,
    /// The Rust `u64` type.
    U64,
    /// The Rust `u128` type.
    U128,
    /// The Rust `usize` type.
    Usize,
    /// The Rust `f32` type.
    F32,
    /// The Rust `f64` type.
    F64,
    /// The Rust `String` type.
    String,
    /// The `chrono::NaiveDate` type.
    Date,
    /// The `chrono::NaiveTime` type.
    Time,
    /// The `chrono::NaiveDateTime` type.
    DateTime,
    /// The `chrono::DateTime<chrono::Utc>` type.
    Instant,
    /// The `bigdecimal::BigDecimal` type.
    BigDecimal,
}

/// The recursive structural shape of a Rust type.
#[must_use]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum TypeShape {
    /// A built-in scalar type.
    Scalar(ScalarType),
    /// A named model type with statically resolvable metadata.
    Named(NamedTypeRef),
    /// An optional value.
    Optional(TypeRef),
    /// An ordered sequence.
    Sequence(TypeRef),
    /// A set of unique values.
    Set(TypeRef),
    /// A mapping from keys to values.
    Map {
        /// The key type.
        key: TypeRef,
        /// The value type.
        value: TypeRef,
    },
    /// A fixed-length array.
    Array {
        /// The element type.
        element: TypeRef,
        /// The number of elements.
        length: usize,
    },
    /// A type intentionally left structurally uninterpreted by the metadata
    /// system.
    Opaque,
}

/// Metadata exposed by types whose structure can be described statically.
pub trait HasTypeShape: 'static {
    /// The recursive shape of this type.
    const TYPE_SHAPE: TypeShape;

    /// Capabilities supported by this type's outermost layer.
    const CAPABILITIES: TypeCapabilities;
}

/// A small, copyable reference to a type's static shape metadata.
#[must_use]
#[derive(Clone, Copy, Debug)]
pub struct TypeRef {
    /// A function that returns the fully qualified name of the referenced
    /// type.
    type_name: fn() -> &'static str,
    /// A function that returns the referenced type's structural shape.
    shape: fn() -> TypeShape,
    /// The capabilities of the referenced type's outermost structural layer.
    capabilities: TypeCapabilities,
}

impl TypeRef {
    /// Creates a reference to the static metadata for `T`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type that implements [`HasTypeShape`].
    ///
    /// # Returns
    ///
    /// A reference to `T`'s static type shape metadata.
    #[inline]
    pub const fn of<T: HasTypeShape>() -> Self {
        Self {
            type_name: type_name::<T>,
            shape: type_shape_of::<T>,
            capabilities: T::CAPABILITIES,
        }
    }

    /// Creates an opaque reference for `T` without requiring [`HasTypeShape`].
    ///
    /// The resulting reference retains `T`'s Rust type name while exposing only
    /// [`TypeShape::Opaque`] to structural queries.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The `'static` Rust type represented by the opaque reference.
    ///
    /// # Returns
    ///
    /// An opaque reference that retains `T`'s type name.
    #[inline]
    pub const fn opaque<T: 'static>() -> Self {
        Self {
            type_name: type_name::<T>,
            shape: opaque_type_shape,
            capabilities: TypeCapabilities::NONE,
        }
    }

    /// Returns the recursive shape represented by this reference.
    ///
    /// # Returns
    ///
    /// The structural shape represented by this reference.
    #[inline(always)]
    pub fn shape(self) -> TypeShape {
        (self.shape)()
    }

    /// Returns Rust's fully qualified name for the referenced type.
    ///
    /// # Returns
    ///
    /// The fully qualified Rust type name represented by this reference.
    #[must_use]
    #[inline(always)]
    pub fn type_name(self) -> &'static str {
        (self.type_name)()
    }

    /// Returns the capabilities supported by the referenced type's outermost
    /// structural layer.
    ///
    /// # Returns
    ///
    /// The capabilities supported by the outermost structural layer.
    #[inline(always)]
    pub const fn capabilities(self) -> TypeCapabilities {
        self.capabilities
    }

    /// Removes one outer `Option` layer, leaving other shapes unchanged.
    ///
    /// # Returns
    ///
    /// The inner reference when the outer shape is optional; otherwise, this
    /// reference unchanged.
    #[inline]
    pub fn strip_optional(self) -> Self {
        match self.shape() {
            TypeShape::Optional(inner) => inner,
            _ => self,
        }
    }

    /// Resolves an outer named type after removing one optional layer, if
    /// present.
    ///
    /// # Returns
    ///
    /// `Some` with the named type's metadata when the resulting shape is named
    /// and has a resolver; otherwise, `None`.
    #[must_use]
    #[inline]
    pub fn named_metadata(self) -> Option<&'static TypeMetadata> {
        match self.strip_optional().shape() {
            TypeShape::Named(named) => named.metadata(),
            _ => None,
        }
    }
}

/// Returns the shape associated with `T` for use in [`TypeRef`].
///
/// # Type Parameters
///
/// * `T` - The type that implements [`HasTypeShape`].
///
/// # Returns
///
/// The static shape declared by `T`.
#[inline]
fn type_shape_of<T: HasTypeShape>() -> TypeShape {
    T::TYPE_SHAPE
}

/// Returns the intentionally uninterpreted shape associated with opaque `T`.
///
/// # Returns
///
/// [`TypeShape::Opaque`].
#[inline]
fn opaque_type_shape() -> TypeShape {
    TypeShape::Opaque
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
impl HasTypeShape for chrono::NaiveDate {
    const TYPE_SHAPE: TypeShape = TypeShape::Scalar(ScalarType::Date);
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::TEMPORAL;
}

#[cfg(feature = "chrono")]
impl HasTypeShape for chrono::NaiveTime {
    const TYPE_SHAPE: TypeShape = TypeShape::Scalar(ScalarType::Time);
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::TEMPORAL;
}

#[cfg(feature = "chrono")]
impl HasTypeShape for chrono::NaiveDateTime {
    const TYPE_SHAPE: TypeShape = TypeShape::Scalar(ScalarType::DateTime);
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::TEMPORAL;
}

#[cfg(feature = "chrono")]
impl HasTypeShape for chrono::DateTime<chrono::Utc> {
    const TYPE_SHAPE: TypeShape = TypeShape::Scalar(ScalarType::Instant);
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::TEMPORAL;
}

#[cfg(feature = "big-decimal")]
impl HasTypeShape for bigdecimal::BigDecimal {
    const TYPE_SHAPE: TypeShape = TypeShape::Scalar(ScalarType::BigDecimal);
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::DECIMAL;
}

impl<T: HasTypeShape> HasTypeShape for Option<T> {
    const TYPE_SHAPE: TypeShape = TypeShape::Optional(TypeRef::of::<T>());
    const CAPABILITIES: TypeCapabilities = T::CAPABILITIES;
}

impl<T: HasTypeShape> HasTypeShape for Vec<T> {
    const TYPE_SHAPE: TypeShape = TypeShape::Sequence(TypeRef::of::<T>());
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::SEQUENCE;
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
    const CAPABILITIES: TypeCapabilities =
        TypeCapabilities::SEQUENCE.union(TypeCapabilities::ARRAY);
}
