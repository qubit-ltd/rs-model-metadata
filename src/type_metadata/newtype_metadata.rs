// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Metadata for single-field tuple newtypes.

use crate::field_metadata::FieldMetadata;

/// Metadata for a single-field tuple newtype.
#[must_use]
#[derive(Clone, Copy, Debug)]
pub struct NewtypeMetadata {
    /// The sole inner field.
    field: FieldMetadata,
}

impl NewtypeMetadata {
    /// Creates newtype metadata from its sole inner field.
    ///
    /// # Parameters
    ///
    /// * `field` - Metadata for the newtype's sole inner field.
    ///
    /// # Returns
    ///
    /// Immutable metadata for the newtype.
    #[inline]
    pub const fn new(field: FieldMetadata) -> Self {
        Self { field }
    }

    /// Returns the sole inner field.
    ///
    /// # Returns
    ///
    /// Metadata for the newtype's sole inner field.
    #[must_use]
    #[inline(always)]
    pub const fn field(self) -> FieldMetadata {
        self.field
    }
}
