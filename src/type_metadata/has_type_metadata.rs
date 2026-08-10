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
