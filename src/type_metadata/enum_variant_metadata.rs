// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Domain metadata overlays for individual reflected enum variants.

use qubit_reflect::VariantDefinitionDescriptor;
use qubit_reflect::VariantDescriptor;

use crate::FieldMetadata;

/// Immutable domain metadata for one enum variant.
#[derive(Clone, Copy, Debug)]
pub struct EnumVariantMetadata {
    /// The reflection descriptor that defines the variant.
    reflect: Option<&'static VariantDescriptor>,
    /// The source declaration for a generic enum overlay.
    definition: Option<&'static VariantDefinitionDescriptor>,
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
            reflect: Some(reflect),
            definition: None,
            canonical_name,
            serialized_name,
            deserialized_name,
            fields,
            default,
        }
    }

    /// Creates an overlay for one generic enum declaration variant.
    #[doc(hidden)]
    #[must_use]
    pub(crate) const fn from_definition(
        definition: &'static VariantDefinitionDescriptor,
        canonical_name: &'static str,
        serialized_name: &'static str,
        deserialized_name: &'static str,
        fields: &'static [FieldMetadata],
        default: bool,
    ) -> Self {
        Self {
            reflect: None,
            definition: Some(definition),
            canonical_name,
            serialized_name,
            deserialized_name,
            fields,
            default,
        }
    }

    /// Returns the underlying structural descriptor.
    #[must_use]
    #[inline(always)]
    pub const fn reflect(&self) -> Option<&'static VariantDescriptor> {
        self.reflect
    }

    /// Returns the generic source declaration variant, when present.
    #[must_use]
    pub const fn definition(&self) -> Option<&'static VariantDefinitionDescriptor> {
        self.definition
    }

    /// Returns the source declaration index.
    #[must_use]
    #[inline(always)]
    pub const fn index(&self) -> usize {
        match (self.reflect, self.definition) {
            (Some(reflect), _) => reflect.index(),
            (_, Some(definition)) => definition.index(),
            _ => unreachable!(),
        }
    }

    /// Returns the immutable Rust identifier.
    #[must_use]
    #[inline(always)]
    pub const fn rust_name(&self) -> &'static str {
        match (self.reflect, self.definition) {
            (Some(reflect), _) => reflect.rust_name(),
            (_, Some(definition)) => definition.rust_name(),
            _ => unreachable!(),
        }
    }

    /// Returns the canonical model name.
    #[must_use]
    #[inline(always)]
    pub const fn canonical_name(&self) -> &'static str {
        self.canonical_name
    }

    /// Returns the serialization name.
    #[must_use]
    #[inline(always)]
    pub const fn serialized_name(&self) -> &'static str {
        self.serialized_name
    }

    /// Returns the deserialization name.
    #[must_use]
    #[inline(always)]
    pub const fn deserialized_name(&self) -> &'static str {
        self.deserialized_name
    }

    /// Returns payload field overlays in source order.
    #[must_use]
    #[inline(always)]
    pub const fn fields(&self) -> &'static [FieldMetadata] {
        self.fields
    }

    /// Finds a named payload field by query name.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&'static FieldMetadata> {
        self.fields.iter().find(|field| field.name() == Some(name))
    }

    /// Returns a payload field by source index.
    #[must_use]
    pub fn field_at(&self, index: usize) -> Option<&'static FieldMetadata> {
        self.fields.get(index)
    }

    /// Returns whether this is the default variant.
    #[must_use]
    #[inline(always)]
    pub const fn is_default(&self) -> bool {
        self.default
    }
}
