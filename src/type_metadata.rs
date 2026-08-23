// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable metadata for named Rust model types.

#[path = "type_metadata/enum_metadata.rs"]
mod enum_metadata;
#[path = "type_metadata/enum_variant_metadata.rs"]
mod enum_variant_metadata;
#[path = "type_metadata/has_type_metadata.rs"]
mod has_type_metadata;
#[path = "type_metadata/named_type_ref.rs"]
mod named_type_ref;
#[path = "type_metadata/newtype_metadata.rs"]
mod newtype_metadata;
#[path = "type_metadata/struct_metadata.rs"]
mod struct_metadata;
#[path = "type_metadata/type_identity.rs"]
mod type_identity;
#[path = "type_metadata/type_kind.rs"]
mod type_kind;

pub use self::enum_metadata::EnumMetadata;
pub use self::enum_variant_metadata::EnumVariantMetadata;
pub use self::has_type_metadata::HasTypeMetadata;
pub use self::has_type_metadata::metadata_of;
pub use self::named_type_ref::NamedTypeRef;
pub use self::newtype_metadata::NewtypeMetadata;
pub use self::struct_metadata::StructMetadata;
pub use self::type_identity::TypeIdentity;
pub use self::type_kind::TypeKind;
use crate::attribute::AttributeMetadata;
use crate::field_metadata::FieldMetadata;
use crate::model_id::ModelId;

/// Immutable metadata for a named model type.
#[must_use]
#[derive(Clone, Copy, Debug)]
pub struct TypeMetadata {
    /// The stable identifier of the model type.
    id: ModelId,
    /// The runtime identity of the model type.
    identity: TypeIdentity,
    /// The structural form of the model type.
    kind: TypeKind,
    /// The model-level attributes in declaration order.
    attributes: &'static [AttributeMetadata],
}

impl TypeMetadata {
    /// Creates type metadata from a stable ID, identity, structural kind, and
    /// attributes.
    ///
    /// # Parameters
    ///
    /// * `id` - The stable identifier of the model type.
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
        id: ModelId,
        identity: TypeIdentity,
        kind: TypeKind,
        attributes: &'static [AttributeMetadata],
    ) -> Self {
        validate_type_attributes(kind, attributes);
        Self {
            id,
            identity,
            kind,
            attributes,
        }
    }

    /// Returns the stable identifier of this model type.
    #[must_use]
    #[inline(always)]
    pub const fn id(&self) -> ModelId {
        self.id
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
const fn validate_type_attributes(kind: TypeKind, attributes: &'static [AttributeMetadata]) {
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
                assert!(primary_key_count == 1, "a model can have at most one primary key");
                validate_primary_key_fields(primary_key, fields);
            }
            AttributeMetadata::Unique(unique) => validate_unique_fields(unique, fields),
            AttributeMetadata::Index(index_metadata) => validate_index_fields(index_metadata.fields(), fields),
            AttributeMetadata::Key(key) => validate_key_fields(key.fields(), fields),
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
            | AttributeMetadata::Generator(_) => {
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
const fn validate_unique_fields(unique: crate::attribute::UniqueMetadata, fields: &'static [FieldMetadata]) {
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
const fn validate_index_fields(names: &'static [&'static str], fields: &'static [FieldMetadata]) {
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
const fn validate_key_fields(names: &'static [&'static str], fields: &'static [FieldMetadata]) {
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
const fn contains_field(fields: &'static [FieldMetadata], name: &'static str) -> bool {
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
