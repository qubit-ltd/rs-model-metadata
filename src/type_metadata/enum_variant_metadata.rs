// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Metadata for fieldless enum variants.

/// Metadata for a fieldless enum variant.
#[must_use]
#[derive(Clone, Copy, Debug)]
pub struct EnumVariantMetadata {
    /// The variant's declaration ordinal.
    ordinal: usize,
    /// The variant's normalized name.
    name: &'static str,
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
        Self { ordinal, name }
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
}
