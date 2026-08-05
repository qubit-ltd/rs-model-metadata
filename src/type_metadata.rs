// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow multiple-public-types
//! Immutable metadata for named Rust model types.

use core::any::{
    TypeId,
    type_name,
};

use crate::attribute::AttributeMetadata;
use crate::field_metadata::FieldMetadata;
use crate::type_shape::HasTypeShape;

/// Metadata exposed by a named type whose structure can be described
/// statically.
pub trait HasTypeMetadata: HasTypeShape {
    /// Returns this type's immutable static metadata.
    ///
    /// # Returns
    ///
    /// The immutable metadata registered for the implementing type.
    fn type_metadata() -> &'static TypeMetadata;
}

/// Returns the immutable static metadata associated with `T`.
///
/// # Type Parameters
///
/// * `T` - The named model type whose metadata is requested.
///
/// # Returns
///
/// The immutable metadata registered for `T`.
#[inline(always)]
pub fn metadata_of<T: HasTypeMetadata>() -> &'static TypeMetadata {
    T::type_metadata()
}

/// A runtime identity for a Rust type, with its fully qualified name retained
/// for display.
///
/// This identity is local to the Rust process/build that produced it. It is
/// suitable for in-memory metadata lookup, but must not be persisted or used
/// as a stable cross-process identifier.
#[derive(Clone, Copy)]
pub struct TypeIdentity {
    /// A function that returns the runtime [`TypeId`] for the represented
    /// type.
    type_id: fn() -> TypeId,
    /// A function that returns the fully qualified name of the represented
    /// type.
    type_name: fn() -> &'static str,
}

impl TypeIdentity {
    /// Creates the identity associated with `T`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The `'static` Rust type represented by the identity.
    ///
    /// # Returns
    ///
    /// A type identity that can be compared, hashed, and displayed.
    #[inline]
    pub const fn of<T: 'static>() -> Self {
        Self {
            type_id: type_id_of::<T>,
            type_name: type_name::<T>,
        }
    }

    /// Returns Rust's runtime identity for this type.
    ///
    /// # Returns
    ///
    /// The runtime [`TypeId`] for this type.
    #[must_use]
    #[inline(always)]
    pub fn type_id(self) -> TypeId {
        (self.type_id)()
    }

    /// Returns Rust's fully qualified name for this type.
    ///
    /// # Returns
    ///
    /// The fully qualified Rust type name.
    #[must_use]
    #[inline(always)]
    pub fn type_name(self) -> &'static str {
        (self.type_name)()
    }
}

impl core::fmt::Debug for TypeIdentity {
    /// Formats the identity with its fully qualified type name.
    ///
    /// # Parameters
    ///
    /// * `formatter` - The formatter receiving the debug representation.
    ///
    /// # Returns
    ///
    /// `Ok(())` when formatting succeeds; otherwise, the formatter's error.
    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        formatter
            .debug_tuple("TypeIdentity")
            .field(&self.type_name())
            .finish()
    }
}

impl PartialEq for TypeIdentity {
    /// Compares identities using Rust's [`TypeId`].
    ///
    /// # Parameters
    ///
    /// * `other` - The identity to compare with this identity.
    ///
    /// # Returns
    ///
    /// `true` when both identities represent the same Rust type; otherwise,
    /// `false`.
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.type_id() == other.type_id()
    }
}

impl Eq for TypeIdentity {}

impl core::hash::Hash for TypeIdentity {
    /// Hashes Rust's [`TypeId`] for this type.
    ///
    /// # Parameters
    ///
    /// * `state` - The hasher receiving this identity's hash.
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.type_id().hash(state);
    }
}

/// Returns the runtime type identity associated with `T`.
///
/// # Type Parameters
///
/// * `T` - The `'static` Rust type whose runtime identity is requested.
///
/// # Returns
///
/// Rust's [`TypeId`] for `T`.
#[inline]
fn type_id_of<T: 'static>() -> TypeId {
    TypeId::of::<T>()
}

/// A static reference to metadata for a named model type.
#[must_use]
#[derive(Clone, Copy, Debug)]
pub struct NamedTypeRef {
    /// The runtime identity of the named type.
    identity: TypeIdentity,
    /// The resolver for metadata in the current model set, when available.
    metadata: Option<fn() -> &'static TypeMetadata>,
}

impl NamedTypeRef {
    /// Creates a resolvable named-type reference for `T`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type that implements [`HasTypeMetadata`].
    ///
    /// # Returns
    ///
    /// A reference containing `T`'s identity and metadata resolver.
    #[inline]
    pub const fn of<T: HasTypeMetadata>() -> Self {
        Self {
            identity: TypeIdentity::of::<T>(),
            metadata: Some(metadata_of::<T>),
        }
    }

    /// Creates a named-type reference from an identity and metadata resolver.
    ///
    /// # Parameters
    ///
    /// * `identity` - The identity of the referenced named type.
    /// * `metadata` - A function that returns the referenced type's metadata.
    ///
    /// # Returns
    ///
    /// A resolvable named-type reference.
    #[inline]
    pub const fn new(
        identity: TypeIdentity,
        metadata: fn() -> &'static TypeMetadata,
    ) -> Self {
        Self {
            identity,
            metadata: Some(metadata),
        }
    }

    /// Creates a named-type reference that cannot resolve metadata in this
    /// model set.
    ///
    /// # Parameters
    ///
    /// * `identity` - The identity of the referenced named type.
    ///
    /// # Returns
    ///
    /// An unresolved named-type reference.
    #[inline]
    pub const fn unresolved(identity: TypeIdentity) -> Self {
        Self {
            identity,
            metadata: None,
        }
    }

    /// Returns the runtime identity of the named type.
    ///
    /// # Returns
    ///
    /// The runtime identity of the named type.
    #[inline(always)]
    pub const fn identity(self) -> TypeIdentity {
        self.identity
    }

    /// Returns metadata for the named type, or `None` when no resolver is
    /// available.
    ///
    /// # Returns
    ///
    /// `Some` with the static metadata when a resolver is available; otherwise,
    /// `None`.
    #[must_use]
    #[inline(always)]
    pub fn metadata(self) -> Option<&'static TypeMetadata> {
        self.metadata.map(|resolve| resolve())
    }
}

/// Immutable metadata for a named model type.
#[must_use]
#[derive(Clone, Copy, Debug)]
pub struct TypeMetadata {
    /// The runtime identity of the model type.
    identity: TypeIdentity,
    /// The structural form of the model type.
    kind: TypeKind,
    /// The model-level attributes in declaration order.
    attributes: &'static [AttributeMetadata],
}

impl TypeMetadata {
    /// Creates type metadata from an identity, structural kind, and attributes.
    ///
    /// # Parameters
    ///
    /// * `identity` - The runtime identity of the model type.
    /// * `kind` - The structural form of the model type.
    /// * `attributes` - The model-level attributes to validate and retain.
    ///
    /// # Returns
    ///
    /// Validated immutable metadata for the model type.
    ///
    /// # Panics
    ///
    /// Panics when model-level attributes have invalid scopes, duplicate
    /// singleton declarations, or references to unknown struct fields.
    #[inline]
    pub const fn new(
        identity: TypeIdentity,
        kind: TypeKind,
        attributes: &'static [AttributeMetadata],
    ) -> Self {
        validate_type_attributes(kind, attributes);
        Self {
            identity,
            kind,
            attributes,
        }
    }

    /// Returns the runtime identity of this model type.
    ///
    /// # Returns
    ///
    /// The runtime identity stored in this metadata.
    #[inline(always)]
    pub const fn identity(&self) -> TypeIdentity {
        self.identity
    }

    /// Returns the structural kind of this model type.
    ///
    /// # Returns
    ///
    /// The structural kind stored in this metadata.
    #[inline(always)]
    pub const fn kind(&self) -> TypeKind {
        self.kind
    }

    /// Returns all model-level metadata attributes.
    ///
    /// # Returns
    ///
    /// The model-level attributes in their declaration order.
    #[must_use]
    #[inline(always)]
    pub const fn attributes(&self) -> &'static [AttributeMetadata] {
        self.attributes
    }

    /// Returns the struct fields, or an empty slice for non-struct model kinds.
    ///
    /// # Returns
    ///
    /// The named fields for a struct, or an empty slice for an enum or newtype.
    #[must_use]
    #[inline]
    pub const fn struct_fields(&self) -> &'static [FieldMetadata] {
        match self.kind {
            TypeKind::Struct(metadata) => metadata.fields(),
            TypeKind::Enum(_) | TypeKind::Newtype(_) => &[],
        }
    }
}

/// The structural form of a named model type.
#[must_use]
#[derive(Clone, Copy, Debug)]
pub enum TypeKind {
    /// A type with named fields.
    Struct(StructMetadata),
    /// A fieldless enum.
    Enum(EnumMetadata),
    /// A tuple newtype with one inner field.
    Newtype(NewtypeMetadata),
}

/// Metadata for a struct with named fields.
#[must_use]
#[derive(Clone, Copy, Debug)]
pub struct StructMetadata {
    /// The fields in declaration order.
    fields: &'static [FieldMetadata],
}

impl StructMetadata {
    /// Creates struct metadata from fields in declaration order.
    ///
    /// # Parameters
    ///
    /// * `fields` - The named fields in declaration order.
    ///
    /// # Returns
    ///
    /// Validated immutable struct metadata.
    ///
    /// # Panics
    ///
    /// Panics when a field ordinal is not contiguous, a field name is empty,
    /// or two fields have the same name.
    #[inline]
    pub const fn new(fields: &'static [FieldMetadata]) -> Self {
        validate_struct_fields(fields);
        Self { fields }
    }

    /// Returns fields in declaration order.
    ///
    /// # Returns
    ///
    /// The named fields in declaration order.
    #[must_use]
    #[inline(always)]
    pub const fn fields(self) -> &'static [FieldMetadata] {
        self.fields
    }
}

/// Validates declaration order and names for a struct's fields.
///
/// # Parameters
///
/// * `fields` - The fields to validate in declaration order.
///
/// # Panics
///
/// Panics when a field ordinal is not contiguous, a field name is empty, or
/// two fields have the same name.
const fn validate_struct_fields(fields: &'static [FieldMetadata]) {
    let mut index = 0;
    while index < fields.len() {
        assert!(
            fields[index].ordinal() == index,
            "field ordinals must match declaration order"
        );
        assert!(
            !fields[index].name().is_empty(),
            "field names cannot be empty"
        );
        let mut previous = 0;
        while previous < index {
            assert!(
                !str_eq(fields[index].name(), fields[previous].name()),
                "struct fields cannot have duplicate names"
            );
            previous += 1;
        }
        index += 1;
    }
}

/// Validates model-level attribute scopes, cardinality, and field references.
///
/// # Parameters
///
/// * `kind` - The model structure used to validate field references.
/// * `attributes` - The model-level attributes to validate.
///
/// # Panics
///
/// Panics when a field-level attribute is used at model scope, a singleton
/// declaration is duplicated, or an attribute references an unknown field.
const fn validate_type_attributes(
    kind: TypeKind,
    attributes: &'static [AttributeMetadata],
) {
    let fields = match kind {
        TypeKind::Struct(metadata) => metadata.fields(),
        TypeKind::Enum(_) | TypeKind::Newtype(_) => &[],
    };
    let mut primary_key_count = 0;
    let mut ownership_count = 0;
    let mut index = 0;
    while index < attributes.len() {
        match attributes[index] {
            AttributeMetadata::PrimaryKey(primary_key) => {
                primary_key_count += 1;
                assert!(
                    primary_key_count == 1,
                    "a model can have at most one primary key"
                );
                validate_primary_key_fields(primary_key, fields);
            }
            AttributeMetadata::Unique(unique) => {
                validate_unique_fields(unique, fields)
            }
            AttributeMetadata::Index(index_metadata) => {
                validate_index_fields(index_metadata.fields(), fields)
            }
            AttributeMetadata::Key(key) => {
                validate_key_fields(key.fields(), fields)
            }
            AttributeMetadata::Ownership(_) => {
                ownership_count += 1;
                assert!(
                    ownership_count == 1,
                    "a model can have at most one ownership declaration"
                );
            }
            AttributeMetadata::Text(_)
            | AttributeMetadata::Sequence(_)
            | AttributeMetadata::Map(_)
            | AttributeMetadata::Temporal(_)
            | AttributeMetadata::Decimal(_)
            | AttributeMetadata::Element(_)
            | AttributeMetadata::Reference(_)
            | AttributeMetadata::LookupRelation(_)
            | AttributeMetadata::Codec(_)
            | AttributeMetadata::Generator(_)
            | AttributeMetadata::Sensitive(_) => {
                panic!("field-level attributes are not valid at model scope")
            }
        }
        index += 1;
    }
}

/// Validates that every primary-key field is declared by the model.
///
/// # Parameters
///
/// * `primary_key` - The primary-key definition to validate.
/// * `fields` - The fields declared by the model.
///
/// # Panics
///
/// Panics when the primary key references an unknown model field.
const fn validate_primary_key_fields(
    primary_key: crate::attribute::PrimaryKeyMetadata,
    fields: &'static [FieldMetadata],
) {
    let fields_to_validate = primary_key.fields();
    let mut index = 0;
    while index < fields_to_validate.len() {
        assert!(
            contains_field(fields, fields_to_validate[index].name()),
            "primary key references an unknown model field"
        );
        index += 1;
    }
}

/// Validates that every unique-constraint field is declared by the model.
///
/// # Parameters
///
/// * `unique` - The unique constraint to validate.
/// * `fields` - The fields declared by the model.
///
/// # Panics
///
/// Panics when the unique constraint references an unknown model field.
const fn validate_unique_fields(
    unique: crate::attribute::UniqueMetadata,
    fields: &'static [FieldMetadata],
) {
    let fields_to_validate = unique.fields();
    let mut index = 0;
    while index < fields_to_validate.len() {
        assert!(
            contains_field(fields, fields_to_validate[index].name()),
            "unique constraint references an unknown model field"
        );
        index += 1;
    }
}

/// Validates that every index field is declared by the model.
///
/// # Parameters
///
/// * `names` - The field names referenced by the index.
/// * `fields` - The fields declared by the model.
///
/// # Panics
///
/// Panics when the index references an unknown model field.
const fn validate_index_fields(
    names: &'static [&'static str],
    fields: &'static [FieldMetadata],
) {
    let mut index = 0;
    while index < names.len() {
        assert!(
            contains_field(fields, names[index]),
            "index references an unknown model field"
        );
        index += 1;
    }
}

/// Validates that every logical-key field is declared by the model.
///
/// # Parameters
///
/// * `names` - The field names referenced by the logical key.
/// * `fields` - The fields declared by the model.
///
/// # Panics
///
/// Panics when the logical key references an unknown model field.
const fn validate_key_fields(
    names: &'static [&'static str],
    fields: &'static [FieldMetadata],
) {
    let mut index = 0;
    while index < names.len() {
        assert!(
            contains_field(fields, names[index]),
            "logical key references an unknown model field"
        );
        index += 1;
    }
}

/// Returns whether `fields` contains a declaration named `name`.
///
/// # Parameters
///
/// * `fields` - The fields to search.
/// * `name` - The field name to find.
///
/// # Returns
///
/// `true` when a field with `name` is present; otherwise, `false`.
const fn contains_field(
    fields: &'static [FieldMetadata],
    name: &'static str,
) -> bool {
    let mut index = 0;
    while index < fields.len() {
        if str_eq(fields[index].name(), name) {
            return true;
        }
        index += 1;
    }
    false
}

/// Compares two static strings without allocating.
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
                !variant.name.is_empty(),
                "enum variant names cannot be empty"
            );
            if variant.ordinal != index {
                panic!("enum variant ordinals must match declaration order");
            }
            let mut previous = 0;
            while previous < index {
                if str_eq(variant.name, variants[previous].name) {
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
            if str_eq(variant.name, name) {
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

/// Metadata for a single-field tuple newtype.
#[must_use]
#[derive(Clone, Copy, Debug)]
pub struct NewtypeMetadata {
    /// The sole inner field.
    field: FieldMetadata,
}

impl NewtypeMetadata {
    /// Creates newtype metadata from its sole inner field.
    ///
    /// # Parameters
    ///
    /// * `field` - Metadata for the newtype's sole inner field.
    ///
    /// # Returns
    ///
    /// Immutable metadata for the newtype.
    #[inline]
    pub const fn new(field: FieldMetadata) -> Self {
        Self { field }
    }

    /// Returns the sole inner field.
    ///
    /// # Returns
    ///
    /// Metadata for the newtype's sole inner field.
    #[must_use]
    #[inline(always)]
    pub const fn field(self) -> FieldMetadata {
        self.field
    }
}
