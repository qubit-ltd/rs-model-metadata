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

use crate::metadata::FieldMetadata;

/// Immutable domain metadata for one enum variant.
///
/// Concrete variants borrow a reflection descriptor; generic definition
/// variants instead borrow a symbolic declaration. Exactly one descriptor is
/// present. Both forms retain source order and naming overlays, but only
/// concrete payload fields have a [`FieldMetadata::location`]. Copying this
/// overlay preserves its descriptors and payload field identities.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Enum;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// #[Enum]
/// enum Reply { Message { text: String }, Pair(u32, u32) }
/// # fn main() {
/// let metadata = TypeMetadata::of::<Reply>();
/// let variants = metadata.as_enum().expect("enum metadata").variants();
/// let message = &variants[0];
/// assert_eq!(message.rust_name(), "Message");
/// let text = message.field("text").expect("named payload field");
/// assert_eq!(text.location().expect("concrete field").variant(), Some(0));
/// assert!(variants[1].field("0").is_none());
/// assert!(variants[1].field_at(0).is_some());
/// assert!(variants[1].field_at(2).is_none());
/// # }
/// ```
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
    #[cfg(feature = "generic")]
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

    /// Returns the concrete structural descriptor, or `None` for a generic
    /// definition variant that has not been specialized.
    #[must_use]
    #[inline(always)]
    pub const fn reflect(&self) -> Option<&'static VariantDescriptor> {
        self.reflect
    }

    /// Returns the symbolic declaration descriptor, or `None` for a concrete
    /// variant, including a specialized generic variant.
    #[must_use]
    #[inline(always)]
    pub const fn definition(&self) -> Option<&'static VariantDefinitionDescriptor> {
        self.definition
    }

    /// Returns the zero-based variant index in the source declaration.
    ///
    /// The index is local to its enum and is unchanged by specialization or
    /// renaming. It is not an explicit Rust discriminant value.
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

    /// Returns payload field overlays in source order, or an empty slice for
    /// a unit variant. Definition fields are symbolic; concrete fields retain
    /// their owner, variant, and field coordinates.
    #[must_use]
    #[inline(always)]
    pub const fn fields(&self) -> &'static [FieldMetadata] {
        self.fields
    }

    /// Finds a named payload field by query name, or returns `None` when
    /// absent.
    ///
    /// Tuple payloads have no query names: use [`Self::field_at`] for their
    /// positions. Numeric strings are not interpreted as tuple indices.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&'static FieldMetadata> {
        self.fields.iter().find(|field| field.name() == Some(name))
    }

    /// Returns a payload field by zero-based source index for either named or
    /// tuple payloads. Returns `None` when the index is outside this variant's
    /// field slice, including every index for a unit variant.
    #[must_use]
    #[inline(always)]
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
