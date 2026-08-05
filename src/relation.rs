// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow multiple-public-types
//! Metadata describing field paths and direct model relations.

use crate::type_metadata::NamedTypeRef;

/// A statically declared sequence of field-name segments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldPath {
    /// The field-name segments in traversal order.
    segments: &'static [&'static str],
}

impl FieldPath {
    /// Creates a field path from statically allocated field-name segments.
    ///
    /// # Parameters
    ///
    /// - `segments`: The field-name segments in traversal order.
    ///
    /// # Returns
    ///
    /// The constructed field path.
    #[must_use]
    #[inline]
    pub const fn new(segments: &'static [&'static str]) -> Self {
        Self { segments }
    }

    /// Returns the path segments in traversal order.
    ///
    /// # Returns
    ///
    /// The statically allocated field-name segments.
    #[must_use]
    #[inline(always)]
    pub const fn segments(self) -> &'static [&'static str] {
        self.segments
    }

    /// Returns whether this path contains no field segments.
    ///
    /// # Returns
    ///
    /// `true` when the path is empty; otherwise `false`.
    #[must_use]
    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.segments.is_empty()
    }
}

/// A direct reference from a field to a target model field.
#[derive(Clone, Copy, Debug)]
pub struct ReferenceMetadata {
    /// The named model containing the referenced field.
    target: NamedTypeRef,
    /// The field path within the target model.
    target_field: FieldPath,
    /// Whether the referenced record must exist.
    must_exist: bool,
    /// An optional field path that must have the same value.
    same_as: Option<FieldPath>,
}

impl ReferenceMetadata {
    /// Creates direct-reference metadata for a named target model and field
    /// path.
    ///
    /// # Parameters
    ///
    /// - `target`: The named model containing the referenced field.
    /// - `target_field`: The field path within the target model.
    /// - `must_exist`: Whether the referenced record must exist.
    /// - `same_as`: An optional field path that must have the same value.
    ///
    /// # Returns
    ///
    /// The constructed direct-reference metadata.
    #[must_use]
    #[inline]
    pub const fn new(
        target: NamedTypeRef,
        target_field: FieldPath,
        must_exist: bool,
        same_as: Option<FieldPath>,
    ) -> Self {
        Self {
            target,
            target_field,
            must_exist,
            same_as,
        }
    }

    /// Returns the named target model.
    ///
    /// # Returns
    ///
    /// The named model containing the referenced field.
    #[inline(always)]
    pub const fn target(self) -> NamedTypeRef {
        self.target
    }

    /// Returns the field path in the target model.
    ///
    /// # Returns
    ///
    /// The field path within the target model.
    #[must_use]
    #[inline(always)]
    pub const fn target_field(self) -> FieldPath {
        self.target_field
    }

    /// Returns whether the target record must exist.
    ///
    /// # Returns
    ///
    /// `true` when the referenced record must exist; otherwise `false`.
    #[must_use]
    #[inline(always)]
    pub const fn must_exist(self) -> bool {
        self.must_exist
    }

    /// Returns the same-as field path, if this reference is constrained to one.
    ///
    /// # Returns
    ///
    /// `Some` with the constrained field path, or `None` when no same-as path
    /// is configured.
    #[must_use]
    #[inline(always)]
    pub const fn same_as(self) -> Option<FieldPath> {
        self.same_as
    }
}

/// A relation resolved by looking up another model.
#[derive(Clone, Copy, Debug)]
pub struct LookupRelationMetadata {
    /// The named model containing the lookup field.
    target: NamedTypeRef,
    /// The field path used for lookup in the target model.
    target_field: FieldPath,
}

impl LookupRelationMetadata {
    /// Creates lookup-relation metadata for a named target and target field
    /// path.
    ///
    /// # Parameters
    ///
    /// - `target`: The named model containing the lookup field.
    /// - `target_field`: The field path used for lookup in the target model.
    ///
    /// # Returns
    ///
    /// The constructed lookup-relation metadata.
    #[must_use]
    #[inline]
    pub const fn new(target: NamedTypeRef, target_field: FieldPath) -> Self {
        Self {
            target,
            target_field,
        }
    }

    /// Returns the named target model.
    ///
    /// # Returns
    ///
    /// The named model containing the lookup field.
    #[inline(always)]
    pub const fn target(self) -> NamedTypeRef {
        self.target
    }

    /// Returns the target field path used for lookup.
    ///
    /// # Returns
    ///
    /// The field path used for lookup in the target model.
    #[must_use]
    #[inline(always)]
    pub const fn target_field(self) -> FieldPath {
        self.target_field
    }
}

/// Metadata declaring the named model that owns this model.
#[derive(Clone, Copy, Debug)]
pub struct OwnershipMetadata {
    /// The named model that owns the current model.
    owner: NamedTypeRef,
}

impl OwnershipMetadata {
    /// Creates ownership metadata for the owning named model.
    ///
    /// # Parameters
    ///
    /// - `owner`: The named model that owns the current model.
    ///
    /// # Returns
    ///
    /// The constructed ownership metadata.
    #[must_use]
    #[inline]
    pub const fn new(owner: NamedTypeRef) -> Self {
        Self { owner }
    }

    /// Returns the owning named model.
    ///
    /// # Returns
    ///
    /// The named model that owns the current model.
    #[inline(always)]
    pub const fn owner(self) -> NamedTypeRef {
        self.owner
    }
}
