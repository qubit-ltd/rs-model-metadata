// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Metadata contract for named Rust model types.

use crate::type_metadata::TypeMetadata;
use crate::type_shape::HasTypeShape;

/// Metadata exposed by a named type whose structure can be described
/// statically.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::HasTypeMetadata;
/// use qubit_model_metadata::HasTypeShape;
/// use qubit_model_metadata::ModelId;
/// use qubit_model_metadata::StructMetadata;
/// use qubit_model_metadata::TypeCapabilities;
/// use qubit_model_metadata::TypeIdentity;
/// use qubit_model_metadata::TypeKind;
/// use qubit_model_metadata::TypeMetadata;
/// use qubit_model_metadata::TypeShape;
/// use qubit_model_metadata::metadata_of;
///
/// struct Account;
///
/// impl HasTypeShape for Account {
///     const TYPE_SHAPE: TypeShape = TypeShape::Opaque;
///     const CAPABILITIES: TypeCapabilities = TypeCapabilities::NONE;
/// }
///
/// impl HasTypeMetadata for Account {
///     fn type_metadata() -> &'static TypeMetadata {
///         static METADATA: TypeMetadata = TypeMetadata::new(
///             ModelId::new("example.Account"),
///             TypeIdentity::of::<Account>(),
///             TypeKind::Struct(StructMetadata::new(&[])),
///             &[],
///         );
///         &METADATA
///     }
/// }
///
/// assert_eq!(metadata_of::<Account>().id().as_str(), "example.Account");
/// ```
pub trait HasTypeMetadata: HasTypeShape {
    /// Returns this type's immutable static metadata.
    ///
    /// # Returns
    ///
    /// The immutable metadata registered for the implementing type.
    fn type_metadata() -> &'static TypeMetadata;
}

/// Returns the immutable static metadata associated with `T`.
///
/// # Type Parameters
///
/// * `T` - The named model type whose metadata is requested.
///
/// # Returns
///
/// The immutable metadata registered for `T`.
#[inline(always)]
pub fn metadata_of<T: HasTypeMetadata>() -> &'static TypeMetadata {
    T::type_metadata()
}
