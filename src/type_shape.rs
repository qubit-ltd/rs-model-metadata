// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Type-level structural metadata for model fields.

#[path = "type_shape/has_type_shape.rs"]
mod has_type_shape;
#[path = "type_shape/scalar_type.rs"]
mod scalar_type;
#[path = "type_shape/type_capabilities.rs"]
mod type_capabilities;
#[path = "type_shape/type_ref.rs"]
mod type_ref;

pub use self::has_type_shape::HasTypeShape;
pub use self::scalar_type::ScalarType;
pub use self::type_capabilities::TypeCapabilities;
pub use self::type_ref::TypeRef;
use crate::type_metadata::NamedTypeRef;

/// The recursive structural shape of a Rust type.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::HasTypeShape;
/// use qubit_model_metadata::TypeShape;
///
/// assert!(matches!(Vec::<u8>::TYPE_SHAPE, TypeShape::Sequence(_)));
/// assert!(matches!(Option::<u8>::TYPE_SHAPE, TypeShape::Optional(_)));
/// ```
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
