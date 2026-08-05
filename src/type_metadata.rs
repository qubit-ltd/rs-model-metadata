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
    fn type_metadata() -> &'static TypeMetadata;
}

/// Returns the immutable static metadata associated with `T`.
#[must_use]
pub fn metadata_of<T: HasTypeMetadata>() -> &'static TypeMetadata {
    T::type_metadata()
}

/// A stable identity for a Rust type, with its fully qualified name retained
/// for display.
#[derive(Clone, Copy)]
pub struct TypeIdentity {
    type_id: fn() -> TypeId,
    type_name: fn() -> &'static str,
}

impl TypeIdentity {
    /// Creates the identity associated with `T`.
    #[must_use]
    pub const fn of<T: 'static>() -> Self {
        Self {
            type_id: type_id_of::<T>,
            type_name: type_name::<T>,
        }
    }

    /// Returns Rust's runtime identity for this type.
    #[must_use]
    pub fn type_id(self) -> TypeId {
        (self.type_id)()
    }

    /// Returns Rust's fully qualified name for this type.
    #[must_use]
    pub fn type_name(self) -> &'static str {
        (self.type_name)()
    }
}

impl core::fmt::Debug for TypeIdentity {
    /// Formats the identity with its fully qualified type name.
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
    fn eq(&self, other: &Self) -> bool {
        self.type_id() == other.type_id()
    }
}

impl Eq for TypeIdentity {}

impl core::hash::Hash for TypeIdentity {
    /// Hashes Rust's [`TypeId`] for this type.
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.type_id().hash(state);
    }
}

/// Returns the runtime type identity associated with `T`.
fn type_id_of<T: 'static>() -> TypeId {
    TypeId::of::<T>()
}

/// A static reference to metadata for a named model type.
#[derive(Clone, Copy, Debug)]
pub struct NamedTypeRef {
    identity: TypeIdentity,
    metadata: Option<fn() -> &'static TypeMetadata>,
}

impl NamedTypeRef {
    /// Creates a resolvable named-type reference for `T`.
    #[must_use]
    pub const fn of<T: HasTypeMetadata>() -> Self {
        Self {
            identity: TypeIdentity::of::<T>(),
            metadata: Some(metadata_of::<T>),
        }
    }

    /// Creates a named-type reference from an identity and metadata resolver.
    #[must_use]
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
    #[must_use]
    pub const fn unresolved(identity: TypeIdentity) -> Self {
        Self {
            identity,
            metadata: None,
        }
    }

    /// Returns the stable identity of the named type.
    #[must_use]
    pub const fn identity(self) -> TypeIdentity {
        self.identity
    }

    /// Returns metadata for the named type, or `None` when no resolver is
    /// available.
    #[must_use]
    pub fn metadata(self) -> Option<&'static TypeMetadata> {
        self.metadata.map(|resolve| resolve())
    }
}

/// Immutable metadata for a named model type.
#[derive(Clone, Copy, Debug)]
pub struct TypeMetadata {
    identity: TypeIdentity,
    kind: TypeKind,
    attributes: &'static [AttributeMetadata],
}

impl TypeMetadata {
    /// Creates type metadata from an identity, structural kind, and attributes.
    #[must_use]
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

    /// Returns the stable identity of this model type.
    #[must_use]
    pub const fn identity(&self) -> TypeIdentity {
        self.identity
    }

    /// Returns the structural kind of this model type.
    #[must_use]
    pub const fn kind(&self) -> TypeKind {
        self.kind
    }

    /// Returns all model-level metadata attributes.
    #[must_use]
    pub const fn attributes(&self) -> &'static [AttributeMetadata] {
        self.attributes
    }

    /// Returns the struct fields, or an empty slice for non-struct model kinds.
    #[must_use]
    pub const fn struct_fields(&self) -> &'static [FieldMetadata] {
        match self.kind {
            TypeKind::Struct(metadata) => metadata.fields(),
            TypeKind::Enum(_) | TypeKind::Newtype(_) => &[],
        }
    }
}

/// The structural form of a named model type.
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
#[derive(Clone, Copy, Debug)]
pub struct StructMetadata {
    fields: &'static [FieldMetadata],
}

impl StructMetadata {
    /// Creates struct metadata from fields in declaration order.
    #[must_use]
    pub const fn new(fields: &'static [FieldMetadata]) -> Self {
        validate_struct_fields(fields);
        Self { fields }
    }

    /// Returns fields in declaration order.
    #[must_use]
    pub const fn fields(self) -> &'static [FieldMetadata] {
        self.fields
    }
}

/// Validates declaration order and names for a struct's fields.
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
#[derive(Clone, Copy, Debug)]
pub struct EnumMetadata {
    variants: &'static [EnumVariantMetadata],
}

impl EnumMetadata {
    /// Creates enum metadata from variants in declaration order.
    #[must_use]
    pub const fn new(variants: &'static [EnumVariantMetadata]) -> Self {
        Self { variants }
    }

    /// Returns variants in declaration order.
    #[must_use]
    pub const fn variants(self) -> &'static [EnumVariantMetadata] {
        self.variants
    }
}

/// Metadata for a fieldless enum variant.
#[derive(Clone, Copy, Debug)]
pub struct EnumVariantMetadata {
    ordinal: usize,
    name: &'static str,
}

impl EnumVariantMetadata {
    /// Creates variant metadata from its declaration ordinal and normalized
    /// name.
    #[must_use]
    pub const fn new(ordinal: usize, name: &'static str) -> Self {
        Self { ordinal, name }
    }

    /// Returns the declaration ordinal of this variant.
    #[must_use]
    pub const fn ordinal(self) -> usize {
        self.ordinal
    }

    /// Returns the normalized variant name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }
}

/// Metadata for a single-field tuple newtype.
#[derive(Clone, Copy, Debug)]
pub struct NewtypeMetadata {
    field: FieldMetadata,
}

impl NewtypeMetadata {
    /// Creates newtype metadata from its sole inner field.
    #[must_use]
    pub const fn new(field: FieldMetadata) -> Self {
        Self { field }
    }

    /// Returns the sole inner field.
    #[must_use]
    pub const fn field(self) -> FieldMetadata {
        self.field
    }
}
