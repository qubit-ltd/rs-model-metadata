// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow multiple-public-types
//! Read-only attribute and field-path query operations.

use crate::attribute::{
    AttributeKind, AttributeMetadata, IndexMetadata, KeyMetadata, PrimaryKeyMetadata,
    UniqueMetadata,
};
use crate::field_metadata::FieldMetadata;
use crate::relation::{FieldPath, OwnershipMetadata};
use crate::type_metadata::{TypeKind, TypeMetadata};
use crate::type_shape::TypeShape;

/// Provides allocation-free queries over a static metadata attribute slice.
pub trait AttributeQuery {
    /// Returns the complete static attribute slice.
    ///
    /// # Returns
    ///
    /// The attributes exposed by the implementing metadata value in
    /// declaration order.
    #[must_use]
    fn attributes(&self) -> &'static [AttributeMetadata];

    /// Returns the first attribute with `kind`, or `None` when it is absent.
    ///
    /// # Parameters
    ///
    /// - `kind`: The attribute kind to find.
    ///
    /// # Returns
    ///
    /// `Some` with the first matching attribute, or `None` when no attribute
    /// has the requested kind.
    #[must_use]
    fn attribute(&self, kind: AttributeKind) -> Option<&'static AttributeMetadata> {
        self.attributes()
            .iter()
            .find(|attribute| attribute.kind() == kind)
    }

    /// Returns an iterator over every attribute with `kind` in declaration
    /// order.
    ///
    /// # Parameters
    ///
    /// - `kind`: The attribute kind to find.
    ///
    /// # Returns
    ///
    /// An iterator over matching attributes in declaration order. The
    /// iterator is empty when no attribute has the requested kind.
    #[must_use]
    fn attributes_of(
        &self,
        kind: AttributeKind,
    ) -> impl Iterator<Item = &'static AttributeMetadata> {
        self.attributes()
            .iter()
            .filter(move |attribute| attribute.kind() == kind)
    }
}

impl AttributeQuery for TypeMetadata {
    /// Returns the model-level static attribute slice.
    ///
    /// # Returns
    ///
    /// The model-level attributes in declaration order.
    #[inline(always)]
    fn attributes(&self) -> &'static [AttributeMetadata] {
        TypeMetadata::attributes(self)
    }
}

impl AttributeQuery for FieldMetadata {
    /// Returns the field-level static attribute slice.
    ///
    /// # Returns
    ///
    /// The field-level attributes in declaration order.
    #[inline(always)]
    fn attributes(&self) -> &'static [AttributeMetadata] {
        (*self).attributes()
    }
}

/// A typed reason why a field path cannot be resolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldPathResolveError {
    /// The supplied path contains no segments.
    EmptyPath,
    /// A field segment is not present in the current struct metadata.
    FieldNotFound {
        /// The missing field segment.
        segment: &'static str,
    },
    /// A non-final path segment does not refer to a named struct.
    IntermediateNotStruct {
        /// The segment whose field cannot be traversed.
        segment: &'static str,
    },
    /// A named intermediate field has no metadata resolver.
    NamedMetadataUnavailable {
        /// The segment whose named type could not be resolved.
        segment: &'static str,
    },
}

impl TypeMetadata {
    /// Returns the model's named fields in declaration order.
    ///
    /// # Returns
    ///
    /// An iterator over the model's fields in declaration order. The iterator
    /// is empty for non-struct model kinds.
    #[must_use = "iterate over the model fields to inspect them"]
    #[inline(always)]
    pub fn fields(&self) -> impl Iterator<Item = &'static FieldMetadata> {
        self.struct_fields().iter()
    }

    /// Returns the named field with `name`, or `None` when no such field
    /// exists.
    ///
    /// # Parameters
    ///
    /// - `name`: The normalized field name to find.
    ///
    /// # Returns
    ///
    /// `Some` with the matching field metadata, or `None` when the model has
    /// no field with that name.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&'static FieldMetadata> {
        self.fields().find(|field| field.name() == name)
    }

    /// Returns the model's primary-key definition, if present.
    ///
    /// # Returns
    ///
    /// `Some` with the primary-key metadata when declared; otherwise `None`.
    #[must_use]
    pub fn primary_key(&self) -> Option<PrimaryKeyMetadata> {
        self.attributes()
            .iter()
            .find_map(|attribute| match attribute {
                AttributeMetadata::PrimaryKey(primary_key) => Some(*primary_key),
                _ => None,
            })
    }

    /// Returns model unique constraints in declaration order.
    ///
    /// # Returns
    ///
    /// An iterator over model-level unique constraints in declaration order.
    /// The iterator is empty when none are declared.
    #[must_use = "iterate over the declared unique constraints to inspect them"]
    pub fn unique_constraints(&self) -> impl Iterator<Item = UniqueMetadata> {
        self.attributes()
            .iter()
            .filter_map(|attribute| match attribute {
                AttributeMetadata::Unique(unique) => Some(*unique),
                _ => None,
            })
    }

    /// Returns model indexes in declaration order.
    ///
    /// # Returns
    ///
    /// An iterator over model-level indexes in declaration order. The
    /// iterator is empty when none are declared.
    #[must_use = "iterate over the declared indexes to inspect them"]
    pub fn indexes(&self) -> impl Iterator<Item = IndexMetadata> {
        self.attributes()
            .iter()
            .filter_map(|attribute| match attribute {
                AttributeMetadata::Index(index) => Some(*index),
                _ => None,
            })
    }

    /// Returns model logical keys in declaration order.
    ///
    /// # Returns
    ///
    /// An iterator over model-level logical keys in declaration order. The
    /// iterator is empty when none are declared.
    #[must_use = "iterate over the declared logical keys to inspect them"]
    pub fn keys(&self) -> impl Iterator<Item = KeyMetadata> {
        self.attributes()
            .iter()
            .filter_map(|attribute| match attribute {
                AttributeMetadata::Key(key) => Some(*key),
                _ => None,
            })
    }

    /// Returns the model ownership declaration, if present.
    ///
    /// # Returns
    ///
    /// `Some` with the ownership metadata when declared; otherwise `None`.
    #[must_use]
    pub fn ownership(&self) -> Option<OwnershipMetadata> {
        self.attributes()
            .iter()
            .find_map(|attribute| match attribute {
                AttributeMetadata::Ownership(ownership) => Some(*ownership),
                _ => None,
            })
    }

    /// Resolves a statically declared field path through named struct metadata.
    ///
    /// Returns the final field when every segment exists. Returns a typed error
    /// for an empty path, a missing field, an unresolvable named type, or
    /// an intermediate non-struct type.
    ///
    /// # Parameters
    ///
    /// - `path`: The statically declared field path to resolve.
    ///
    /// # Returns
    ///
    /// `Ok` with the final field metadata when every segment resolves.
    ///
    /// # Errors
    ///
    /// Returns [`FieldPathResolveError::EmptyPath`] for an empty path,
    /// [`FieldPathResolveError::FieldNotFound`] for a missing field,
    /// [`FieldPathResolveError::NamedMetadataUnavailable`] when a named
    /// intermediate type has no resolver, or
    /// [`FieldPathResolveError::IntermediateNotStruct`] when traversal would
    /// pass through a non-struct type.
    #[must_use = "inspect the resolved field or handle the resolution error"]
    pub fn resolve_field_path(
        &self,
        path: FieldPath,
    ) -> Result<&'static FieldMetadata, FieldPathResolveError> {
        let segments = path.segments();
        let (first, remaining) = segments
            .split_first()
            .ok_or(FieldPathResolveError::EmptyPath)?;
        self.resolve_field_path_from(first, remaining)
    }

    /// Resolves `segment` and the remaining path from this metadata node.
    ///
    /// # Parameters
    ///
    /// - `segment`: The current field-name segment.
    /// - `remaining`: The remaining field-name segments.
    ///
    /// # Returns
    ///
    /// `Ok` with the final field metadata when the remaining path resolves.
    ///
    /// # Errors
    ///
    /// Returns [`FieldPathResolveError::FieldNotFound`] when `segment` is not
    /// declared, [`FieldPathResolveError::NamedMetadataUnavailable`] when an
    /// intermediate named type has no resolver, or
    /// [`FieldPathResolveError::IntermediateNotStruct`] when traversal would
    /// pass through a non-struct type.
    fn resolve_field_path_from(
        &self,
        segment: &'static str,
        remaining: &'static [&'static str],
    ) -> Result<&'static FieldMetadata, FieldPathResolveError> {
        let field = self
            .field(segment)
            .ok_or(FieldPathResolveError::FieldNotFound { segment })?;
        let Some((next_segment, next_remaining)) = remaining.split_first() else {
            return Ok(field);
        };

        let named = match field.field_type().strip_optional().shape() {
            TypeShape::Named(named) => named,
            _ => {
                return Err(FieldPathResolveError::IntermediateNotStruct { segment });
            }
        };
        let metadata = named
            .metadata()
            .ok_or(FieldPathResolveError::NamedMetadataUnavailable { segment })?;
        match metadata.kind() {
            TypeKind::Struct(_) => metadata.resolve_field_path_from(next_segment, next_remaining),
            TypeKind::Enum(_) | TypeKind::Newtype(_) => {
                Err(FieldPathResolveError::IntermediateNotStruct { segment })
            }
        }
    }
}
