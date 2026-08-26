// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Metadata for enum variants.

use super::enum_variant_kind::EnumVariantKind;
use super::enum_variant_kind::validate_variant_fields;
use crate::field_metadata::FieldMetadata;

/// Metadata for an enum variant and its optional payload fields.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::EnumVariantMetadata;
///
/// let variant = EnumVariantMetadata::new(0, "Active");
/// assert_eq!(variant.name(), "Active");
/// ```
#[must_use]
#[derive(Clone, Copy, Debug)]
pub struct EnumVariantMetadata {
    /// The variant's declaration ordinal.
    ordinal: usize,
    /// The variant's normalized name.
    name: &'static str,
    /// The variant's structural form and payload fields.
    kind: EnumVariantKind,
}

impl EnumVariantMetadata {
    /// Creates variant metadata from its declaration ordinal and normalized
    /// name.
    ///
    /// # Parameters
    ///
    /// * `ordinal` - The variant's zero-based declaration ordinal.
    /// * `name` - The variant's normalized name.
    ///
    /// # Returns
    ///
    /// Immutable metadata for the enum variant.
    #[inline]
    pub const fn new(ordinal: usize, name: &'static str) -> Self {
        Self {
            ordinal,
            name,
            kind: EnumVariantKind::Unit,
        }
    }

    /// Creates metadata for a tuple variant.
    ///
    /// # Parameters
    ///
    /// * `ordinal` - The variant's zero-based declaration ordinal.
    /// * `name` - The variant's normalized name.
    /// * `fields` - Positional fields in declaration order. Metadata producers
    ///   should use decimal ordinals such as `"0"` as field names.
    ///
    /// # Returns
    ///
    /// Immutable metadata for the tuple variant.
    ///
    /// # Panics
    ///
    /// Panics when field ordinals are not contiguous, a field name is empty,
    /// or two field names are duplicated.
    #[inline]
    pub const fn tuple(ordinal: usize, name: &'static str, fields: &'static [FieldMetadata]) -> Self {
        validate_variant_fields(fields);
        Self {
            ordinal,
            name,
            kind: EnumVariantKind::Tuple(fields),
        }
    }

    /// Creates metadata for a struct variant.
    ///
    /// # Parameters
    ///
    /// * `ordinal` - The variant's zero-based declaration ordinal.
    /// * `name` - The variant's normalized name.
    /// * `fields` - Named fields in declaration order.
    ///
    /// # Returns
    ///
    /// Immutable metadata for the struct variant.
    ///
    /// # Panics
    ///
    /// Panics when field ordinals are not contiguous, a field name is empty,
    /// or two field names are duplicated.
    #[inline]
    pub const fn structure(ordinal: usize, name: &'static str, fields: &'static [FieldMetadata]) -> Self {
        validate_variant_fields(fields);
        Self {
            ordinal,
            name,
            kind: EnumVariantKind::Struct(fields),
        }
    }

    /// Returns the declaration ordinal of this variant.
    ///
    /// # Returns
    ///
    /// The variant's zero-based declaration ordinal.
    #[must_use]
    #[inline(always)]
    pub const fn ordinal(self) -> usize {
        self.ordinal
    }

    /// Returns the normalized variant name.
    ///
    /// # Returns
    ///
    /// The normalized variant name.
    #[must_use]
    #[inline(always)]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Returns the variant's structural form.
    ///
    /// # Returns
    ///
    /// The unit, tuple, or struct form and its payload fields.
    #[inline(always)]
    pub const fn kind(self) -> EnumVariantKind {
        self.kind
    }

    /// Returns the variant's payload fields.
    ///
    /// # Returns
    ///
    /// The tuple or struct fields in declaration order, or an empty slice for
    /// a unit variant.
    #[must_use]
    #[inline(always)]
    pub const fn fields(self) -> &'static [FieldMetadata] {
        match self.kind {
            EnumVariantKind::Unit => &[],
            EnumVariantKind::Tuple(fields) | EnumVariantKind::Struct(fields) => fields,
        }
    }
}
