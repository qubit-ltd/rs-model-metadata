// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Probe for generated metadata on nested field types.

use std::marker::PhantomData;

use crate::metadata::HasTypeMetadata;
use crate::metadata::TypeMetadata;

/// Selects generated metadata when a nested field type has a model derive.
#[doc(hidden)]
pub trait NestedTypeMetadataProbe {
    /// Returns the generated metadata when the probed type has one.
    ///
    /// # Returns
    ///
    /// `Some` contains the probed type's metadata; `None` means the type has
    /// no generated metadata.
    fn nested_type_metadata(self) -> Option<&'static TypeMetadata>;
}

impl<T: HasTypeMetadata> NestedTypeMetadataProbe for &&PhantomData<T> {
    /// Returns the metadata generated for `T` by the model derive.
    ///
    /// The double reference selects this implementation only when `T`
    /// implements [`HasTypeMetadata`]. Metadata lookup is static and does not
    /// inspect a runtime value.
    #[inline]
    fn nested_type_metadata(self) -> Option<&'static TypeMetadata> {
        TypeMetadata::try_of::<T>().ok()
    }
}

impl<T> NestedTypeMetadataProbe for &PhantomData<T> {
    /// Reports that `T` has no generated metadata available through this
    /// probe implementation.
    ///
    /// This implementation is selected for types without the
    /// [`HasTypeMetadata`] bound and returns `None` without inspecting a
    /// runtime value.
    #[inline]
    fn nested_type_metadata(self) -> Option<&'static TypeMetadata> {
        None
    }
}
