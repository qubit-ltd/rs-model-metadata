// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Domain metadata indexes for reflected enum variants.

use crate::metadata::EnumVariantMetadata;

/// Metadata for enum variants in source order.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Enum;
/// use qubit_model_metadata::metadata::EnumMetadata;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// #[Enum]
/// enum Reply { Ready, Done }
/// # fn main() {
/// let metadata = TypeMetadata::of::<Reply>();
/// let variants: &EnumMetadata = metadata.as_enum().expect("enum metadata");
/// assert_eq!(variants.variants().len(), 2);
/// assert_eq!(variants.variant("READY").map(|item| item.rust_name()), Some("Ready"));
/// assert_eq!(variants.variant_by_rust_name("Ready").map(|item| item.canonical_name()), Some("READY"));
/// assert_eq!(variants.variant_by_serialized_name("READY").map(|item| item.rust_name()), Some("Ready"));
/// assert!(variants.variant("UNKNOWN").is_none());
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct EnumMetadata {
    /// Variant metadata in declaration order.
    variants: &'static [EnumVariantMetadata],
}

impl EnumMetadata {
    /// Creates enum metadata.
    #[must_use]
    #[inline]
    pub(crate) const fn new(variants: &'static [EnumVariantMetadata]) -> Self {
        Self { variants }
    }

    /// Returns variants in source order.
    ///
    /// # Returns
    ///
    /// The static slice of enum variants in declaration order.
    #[must_use]
    #[inline]
    pub const fn variants(&self) -> &'static [EnumVariantMetadata] {
        self.variants
    }

    /// Finds a variant by canonical model name.
    ///
    /// # Parameters
    ///
    /// * `name` - Exact canonical model name to look up.
    ///
    /// # Returns
    ///
    /// The matching variant, or `None` when no canonical name matches.
    #[must_use]
    pub fn variant(&self, name: &str) -> Option<&'static EnumVariantMetadata> {
        self.variants
            .iter()
            .find(|variant| variant.canonical_name() == name)
    }

    /// Finds a variant by Rust identifier.
    ///
    /// # Parameters
    ///
    /// * `name` - Exact Rust variant identifier to look up.
    ///
    /// # Returns
    ///
    /// The matching variant, or `None` when no Rust identifier matches.
    #[must_use]
    pub fn variant_by_rust_name(&self, name: &str) -> Option<&'static EnumVariantMetadata> {
        self.variants
            .iter()
            .find(|variant| variant.rust_name() == name)
    }

    /// Finds a variant by serialization name.
    ///
    /// # Parameters
    ///
    /// * `name` - Exact name emitted while serializing the variant.
    ///
    /// # Returns
    ///
    /// The matching variant, or `None` when no serialization name matches.
    #[must_use]
    pub fn variant_by_serialized_name(&self, name: &str) -> Option<&'static EnumVariantMetadata> {
        self.variants
            .iter()
            .find(|variant| variant.serialized_name() == name)
    }
}
