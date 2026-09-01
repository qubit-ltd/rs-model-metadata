// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Domain overlay for one reflected enum variant.

use qubit_reflect::VariantDescriptor;

use crate::FieldMetadata;

/// Immutable domain metadata for one enum variant.
#[derive(Clone, Copy, Debug)]
pub struct EnumVariantMetadata {
    /// The reflection descriptor that defines the variant.
    reflect: &'static VariantDescriptor,
    /// The model-level canonical variant name.
    canonical_name: &'static str,
    /// The name emitted while serializing the variant.
    serialized_name: &'static str,
    /// The name accepted while deserializing the variant.
    deserialized_name: &'static str,
    /// Payload field overlays in declaration order.
    fields: &'static [FieldMetadata],
    /// Whether this variant is the model default.
    default: bool,
}

impl EnumVariantMetadata {
    /// Creates an enum-variant overlay.
    #[must_use]
    pub(crate) const fn new(
        reflect: &'static VariantDescriptor,
        canonical_name: &'static str,
        serialized_name: &'static str,
        deserialized_name: &'static str,
        fields: &'static [FieldMetadata],
        default: bool,
    ) -> Self {
        Self {
            reflect,
            canonical_name,
            serialized_name,
            deserialized_name,
            fields,
            default,
        }
    }

    /// Returns the underlying structural descriptor.
    #[must_use]
    pub const fn reflect(&self) -> &'static VariantDescriptor {
        self.reflect
    }

    /// Returns the source declaration index.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.reflect.index()
    }

    /// Returns the immutable Rust identifier.
    #[must_use]
    pub const fn rust_name(&self) -> &'static str {
        self.reflect.rust_name()
    }

    /// Returns the canonical model name.
    #[must_use]
    pub const fn canonical_name(&self) -> &'static str {
        self.canonical_name
    }

    /// Returns the serialization name.
    #[must_use]
    pub const fn serialized_name(&self) -> &'static str {
        self.serialized_name
    }

    /// Returns the deserialization name.
    #[must_use]
    pub const fn deserialized_name(&self) -> &'static str {
        self.deserialized_name
    }

    /// Returns payload field overlays in source order.
    #[must_use]
    pub const fn fields(&self) -> &'static [FieldMetadata] {
        self.fields
    }

    /// Finds a named payload field by query name.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&'static FieldMetadata> {
        self.fields
            .iter()
            .find(|field| field.reflect().query_name() == Some(name))
    }

    /// Returns a payload field by source index.
    #[must_use]
    pub fn field_at(&self, index: usize) -> Option<&'static FieldMetadata> {
        self.fields.get(index)
    }

    /// Returns whether this is the default variant.
    #[must_use]
    pub const fn is_default(&self) -> bool {
        self.default
    }
}
