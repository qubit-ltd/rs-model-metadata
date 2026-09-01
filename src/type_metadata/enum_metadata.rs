// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Domain metadata for reflected enums.

use crate::EnumVariantMetadata;

/// Metadata for enum variants in source order.
#[derive(Clone, Copy, Debug)]
pub struct EnumMetadata {
    /// Variant metadata in declaration order.
    variants: &'static [EnumVariantMetadata],
}

impl EnumMetadata {
    /// Creates enum metadata.
    #[must_use]
    pub(crate) const fn new(variants: &'static [EnumVariantMetadata]) -> Self {
        Self { variants }
    }

    /// Returns variants in source order.
    #[must_use]
    pub const fn variants(&self) -> &'static [EnumVariantMetadata] {
        self.variants
    }

    /// Finds a variant by canonical model name.
    #[must_use]
    pub fn variant(&self, name: &str) -> Option<&'static EnumVariantMetadata> {
        self.variants.iter().find(|variant| variant.canonical_name() == name)
    }

    /// Finds a variant by Rust identifier.
    #[must_use]
    pub fn variant_by_rust_name(&self, name: &str) -> Option<&'static EnumVariantMetadata> {
        self.variants.iter().find(|variant| variant.rust_name() == name)
    }

    /// Finds a variant by serialization name.
    #[must_use]
    pub fn variant_by_serialized_name(&self, name: &str) -> Option<&'static EnumVariantMetadata> {
        self.variants.iter().find(|variant| variant.serialized_name() == name)
    }
}
