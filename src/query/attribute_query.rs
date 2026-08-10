// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Allocation-free queries over static metadata attributes.

use crate::attribute::AttributeKind;
use crate::attribute::AttributeMetadata;
use crate::field_metadata::FieldMetadata;
use crate::type_metadata::TypeMetadata;

/// Provides allocation-free queries over a static metadata attribute slice.
pub trait AttributeQuery {
    /// Returns the complete static attribute slice.
    ///
    /// # Returns
    ///
    /// The attributes exposed by the implementing metadata value in
    /// declaration order.
    #[must_use]
    fn attributes(&self) -> &'static [AttributeMetadata];

    /// Returns the first attribute with `kind`, or `None` when it is absent.
    ///
    /// # Parameters
    ///
    /// - `kind`: The attribute kind to find.
    ///
    /// # Returns
    ///
    /// `Some` with the first matching attribute, or `None` when no attribute
    /// has the requested kind.
    #[must_use]
    fn attribute(
        &self,
        kind: AttributeKind,
    ) -> Option<&'static AttributeMetadata> {
        self.attributes()
            .iter()
            .find(|attribute| attribute.kind() == kind)
    }

    /// Returns an iterator over every attribute with `kind` in declaration
    /// order.
    ///
    /// # Parameters
    ///
    /// - `kind`: The attribute kind to find.
    ///
    /// # Returns
    ///
    /// An iterator over matching attributes in declaration order. The
    /// iterator is empty when no attribute has the requested kind.
    #[must_use]
    fn attributes_of(
        &self,
        kind: AttributeKind,
    ) -> impl Iterator<Item = &'static AttributeMetadata> {
        self.attributes()
            .iter()
            .filter(move |attribute| attribute.kind() == kind)
    }
}

impl AttributeQuery for TypeMetadata {
    /// Returns the model-level static attribute slice.
    ///
    /// # Returns
    ///
    /// The model-level attributes in declaration order.
    #[inline(always)]
    fn attributes(&self) -> &'static [AttributeMetadata] {
        TypeMetadata::attributes(self)
    }
}

impl AttributeQuery for FieldMetadata {
    /// Returns the field-level static attribute slice.
    ///
    /// # Returns
    ///
    /// The field-level attributes in declaration order.
    #[inline(always)]
    fn attributes(&self) -> &'static [AttributeMetadata] {
        (*self).attributes()
    }
}
