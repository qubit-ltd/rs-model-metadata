// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Metadata for fieldless enums.

use crate::type_metadata::EnumVariantMetadata;

/// Metadata for a fieldless enum.
#[must_use]
#[derive(Clone, Copy, Debug)]
pub struct EnumMetadata {
    /// The variants in declaration order.
    variants: &'static [EnumVariantMetadata],
}

impl EnumMetadata {
    /// Creates enum metadata from variants in declaration order.
    ///
    /// # Parameters
    ///
    /// * `variants` - The variants in declaration order.
    ///
    /// # Returns
    ///
    /// Immutable metadata for the fieldless enum.
    ///
    /// # Panics
    ///
    /// Panics when a variant name is empty or duplicated, or when a variant's
    /// ordinal does not match its position in `variants`.
    #[inline]
    pub const fn new(variants: &'static [EnumVariantMetadata]) -> Self {
        let mut index = 0;
        while index < variants.len() {
            let variant = variants[index];
            assert!(
                !variant.name().is_empty(),
                "enum variant names cannot be empty"
            );
            if variant.ordinal() != index {
                panic!("enum variant ordinals must match declaration order");
            }
            let mut previous = 0;
            while previous < index {
                if str_eq(variant.name(), variants[previous].name()) {
                    panic!("enum variant names must be unique");
                }
                previous += 1;
            }
            index += 1;
        }
        Self { variants }
    }

    /// Returns variants in declaration order.
    ///
    /// # Returns
    ///
    /// The enum variants in declaration order.
    #[inline(always)]
    pub const fn variants(self) -> &'static [EnumVariantMetadata] {
        self.variants
    }

    /// Returns the first variant with the supplied normalized name.
    ///
    /// # Parameters
    ///
    /// * `name` - The normalized variant name to search for.
    ///
    /// # Returns
    ///
    /// `Some` with the matching variant, or `None` when no variant has that
    /// name.
    #[must_use]
    pub const fn variant(self, name: &str) -> Option<EnumVariantMetadata> {
        let mut index = 0;
        while index < self.variants.len() {
            let variant = self.variants[index];
            if str_eq(variant.name(), name) {
                return Some(variant);
            }
            index += 1;
        }
        None
    }

    /// Returns the variant declared at `ordinal`.
    ///
    /// # Parameters
    ///
    /// * `ordinal` - The zero-based declaration ordinal to search for.
    ///
    /// # Returns
    ///
    /// `Some` with the matching variant, or `None` when the ordinal is out of
    /// range.
    #[must_use]
    pub const fn variant_at(
        self,
        ordinal: usize,
    ) -> Option<EnumVariantMetadata> {
        if ordinal < self.variants.len() {
            Some(self.variants[ordinal])
        } else {
            None
        }
    }
}

/// Compares two strings without allocating.
///
/// # Parameters
///
/// * `left` - The first string to compare.
/// * `right` - The second string to compare.
///
/// # Returns
///
/// `true` when both strings contain the same bytes; otherwise, `false`.
const fn str_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}
