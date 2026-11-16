// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Domain metadata overlays for individual reflected enum variants.

use std::any::TypeId;

use qubit_reflect::ReflectionContext;
use qubit_reflect::descriptor::VariantDescriptor as BoundVariantDescriptor;

use crate::metadata::FieldMetadata;

/// Immutable domain metadata for one enum variant.
///
/// Concrete variants borrow a reflection descriptor; generic definition
/// variants instead borrow a symbolic declaration. Exactly one descriptor is
/// present. Both forms retain source order and naming overlays, but only
/// concrete payload fields have a [`FieldMetadata::location`]. Copying this
/// overlay preserves its descriptors and payload field identities.
///
/// Obtain variants through [`crate::metadata::TypeMetadata::of`] and
/// [`crate::metadata::EnumMetadata::variants`]; callers do not construct this
/// framework-provided metadata directly.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Enum;
/// use qubit_model_metadata::metadata::EnumVariantMetadata;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// #[Enum]
/// enum Reply { Message { text: String }, Pair(u32, u32) }
/// # fn main() {
/// let metadata = TypeMetadata::of::<Reply>();
/// let variants = metadata.as_enum().expect("enum metadata").variants();
/// let message: &EnumVariantMetadata = &variants[0];
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
    /// Concrete declaring type identity, absent for a generic definition.
    owner_type_id: Option<TypeId>,
    /// Generic definition path that owns a symbolic variant.
    definition_path: Option<&'static str>,
    /// Source variant index.
    source_index: usize,
    /// Source Rust variant name.
    source_rust_name: &'static str,
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
    /// Creates metadata for a concrete variant of a reflected enum.
    ///
    /// The owner identity and payload metadata must describe the same enum
    /// declaration, and `index` must be that variant's zero-based source
    /// position. Names preserve the model's canonical, serialization, and
    /// deserialization overlays; `fields` must remain valid for the program's
    /// lifetime because this metadata stores a static borrow. The `default`
    /// flag records the model-level default designation.
    ///
    /// # Parameters
    ///
    /// * `owner` - Runtime type identity used to resolve the concrete
    ///   reflection descriptor.
    /// * `index` - Zero-based source position of this variant in its enum.
    /// * `rust_name` - Original Rust variant identifier.
    /// * `canonical_name` - Model-level canonical variant name.
    /// * `serialized_name` - Variant name emitted during serialization.
    /// * `deserialized_name` - Variant name accepted during deserialization.
    /// * `fields` - Static payload field metadata in declaration order.
    /// * `default` - Whether this variant is the model's default variant.
    ///
    /// # Returns
    ///
    /// An overlay linked to the concrete owner type and carrying the supplied
    /// variant names, payload fields, and default designation.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn new(
        owner: TypeId,
        index: usize,
        rust_name: &'static str,
        canonical_name: &'static str,
        serialized_name: &'static str,
        deserialized_name: &'static str,
        fields: &'static [FieldMetadata],
        default: bool,
    ) -> Self {
        Self {
            owner_type_id: Some(owner),
            definition_path: None,
            source_index: index,
            source_rust_name: rust_name,
            canonical_name,
            serialized_name,
            deserialized_name,
            fields,
            default,
        }
    }

    /// Creates metadata for a symbolic variant in a generic enum definition.
    ///
    /// The definition path and payload metadata must describe the same
    /// declaration, and `index` must be that variant's zero-based source
    /// position. Names preserve the model's canonical, serialization, and
    /// deserialization overlays; `fields` must remain valid for the program's
    /// lifetime because this metadata stores a static borrow. The `default`
    /// flag records the model-level default designation. Unlike concrete
    /// metadata, this form has no runtime owner identity and cannot resolve a
    /// concrete reflection descriptor until specialized.
    ///
    /// # Parameters
    ///
    /// * `definition_path` - Stable path identifying the generic enum
    ///   declaration.
    /// * `index` - Zero-based source position of this variant in its enum.
    /// * `rust_name` - Original Rust variant identifier.
    /// * `canonical_name` - Model-level canonical variant name.
    /// * `serialized_name` - Variant name emitted during serialization.
    /// * `deserialized_name` - Variant name accepted during deserialization.
    /// * `fields` - Static symbolic payload field metadata in declaration
    ///   order.
    /// * `default` - Whether this variant is the model's default variant.
    ///
    /// # Returns
    ///
    /// An overlay linked to the generic definition path and carrying the
    /// supplied variant names, payload fields, and default designation.
    #[doc(hidden)]
    #[must_use]
    #[cfg(feature = "generic")]
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn from_definition(
        definition_path: &'static str,
        index: usize,
        rust_name: &'static str,
        canonical_name: &'static str,
        serialized_name: &'static str,
        deserialized_name: &'static str,
        fields: &'static [FieldMetadata],
        default: bool,
    ) -> Self {
        Self {
            owner_type_id: None,
            definition_path: Some(definition_path),
            source_index: index,
            source_rust_name: rust_name,
            canonical_name,
            serialized_name,
            deserialized_name,
            fields,
            default,
        }
    }

    /// Returns the concrete structural descriptor, or `None` for a generic
    /// definition variant that has not been specialized.
    ///
    /// # Parameters
    ///
    /// * `context` - Reflection context containing the concrete owner type.
    ///
    /// # Returns
    ///
    /// The descriptor for this variant when its concrete owner is registered
    /// in `context`; otherwise, `None`.
    #[inline]
    pub fn reflect_in<'ctx>(&self, context: &'ctx ReflectionContext) -> Option<BoundVariantDescriptor<'ctx>> {
        let ty = context.find_type_by_id(self.owner_type_id?).ok().flatten()?;
        ty.variants()
            .iter()
            .find(|variant| variant.index() == self.source_index)
    }

    /// Returns the generic definition path for a symbolic variant.
    ///
    /// # Returns
    ///
    /// The owning definition path for a generic variant, or `None` for a
    /// concrete variant.
    pub const fn definition_path(&self) -> Option<&'static str> {
        self.definition_path
    }

    /// Returns the zero-based variant index in the source declaration.
    ///
    /// The index is local to its enum and is unchanged by specialization or
    /// renaming. It is not an explicit Rust discriminant value.
    ///
    /// # Returns
    ///
    /// The zero-based position of this variant in its source enum.
    #[must_use]
    #[inline]
    pub const fn index(&self) -> usize {
        self.source_index
    }

    /// Returns the immutable Rust identifier.
    ///
    /// # Returns
    ///
    /// The variant's source-level Rust identifier.
    #[must_use]
    #[inline]
    pub const fn rust_name(&self) -> &'static str {
        self.source_rust_name
    }

    /// Returns the canonical model name.
    ///
    /// # Returns
    ///
    /// The canonical name used by the model metadata.
    #[must_use]
    #[inline]
    pub const fn canonical_name(&self) -> &'static str {
        self.canonical_name
    }

    /// Returns the serialization name.
    ///
    /// # Returns
    ///
    /// The name emitted when serializing this variant.
    #[must_use]
    #[inline]
    pub const fn serialized_name(&self) -> &'static str {
        self.serialized_name
    }

    /// Returns the deserialization name.
    ///
    /// # Returns
    ///
    /// The name accepted when deserializing this variant.
    #[must_use]
    #[inline]
    pub const fn deserialized_name(&self) -> &'static str {
        self.deserialized_name
    }

    /// Returns payload field overlays in source order, or an empty slice for
    /// a unit variant. Definition fields are symbolic; concrete fields retain
    /// their owner, variant, and field coordinates.
    ///
    /// # Returns
    ///
    /// Payload fields in source declaration order; unit variants return an
    /// empty slice.
    #[must_use]
    #[inline]
    pub const fn fields(&self) -> &'static [FieldMetadata] {
        self.fields
    }

    /// Finds a named payload field by query name, or returns `None` when
    /// absent.
    ///
    /// Tuple payloads have no query names: use [`Self::field_at`] for their
    /// positions. Numeric strings are not interpreted as tuple indices.
    ///
    /// # Parameters
    ///
    /// * `name` - Exact payload field query name.
    ///
    /// # Returns
    ///
    /// The matching named field, or `None` when the name does not match a
    /// named payload field.
    pub fn field(&self, name: &str) -> Option<&'static FieldMetadata> {
        self.fields.iter().find(|field| field.name() == Some(name))
    }

    /// Returns a payload field by zero-based source index for either named or
    /// tuple payloads. Returns `None` when the index is outside this variant's
    /// field slice, including every index for a unit variant.
    ///
    /// # Parameters
    ///
    /// * `index` - Zero-based position in the payload declaration.
    ///
    /// # Returns
    ///
    /// The payload field at that position, or `None` when it is out of range.
    #[inline]
    pub fn field_at(&self, index: usize) -> Option<&'static FieldMetadata> {
        self.fields.get(index)
    }

    /// Returns whether this is the default variant.
    ///
    /// # Returns
    ///
    /// `true` when this variant is the model default; otherwise, `false`.
    #[must_use]
    #[inline]
    pub const fn is_default(&self) -> bool {
        self.default
    }
}
