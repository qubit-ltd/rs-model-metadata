// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Relation endpoint projection used by direct-reference graph validation.

use crate::type_metadata::TypeIdentity;
use crate::type_shape::TypeRef;
use crate::type_shape::TypeShape;

/// A relation endpoint projected from a structural type.
#[derive(Clone, Copy)]
pub(crate) struct RelationProjection {
    /// The leaf type identity, when the structure has one unambiguous value.
    identity: Option<TypeIdentity>,
    /// The leaf type name used in diagnostics.
    type_name: &'static str,
}

impl RelationProjection {
    /// Creates a projection with a leaf identity and diagnostic type name.
    ///
    /// # Parameters
    ///
    /// - `identity`: The leaf type identity, when the structure has one
    ///   unambiguous value.
    /// - `type_name`: The leaf type name used in diagnostics.
    ///
    /// # Returns
    ///
    /// A projection describing one relation endpoint.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn new(identity: Option<TypeIdentity>, type_name: &'static str) -> Self {
        Self { identity, type_name }
    }

    /// Returns whether two projected relation endpoints have the same leaf.
    ///
    /// # Parameters
    ///
    /// - `other`: The projection to compare with this endpoint.
    ///
    /// # Returns
    ///
    /// `true` when both projections have matching leaf identities; otherwise
    /// `false`.
    #[must_use]
    pub(crate) fn is_compatible_with(self, other: Self) -> bool {
        self.identity
            .zip(other.identity)
            .is_some_and(|(left, right)| left == right)
    }

    /// Returns the diagnostic name for this projected endpoint.
    ///
    /// # Returns
    ///
    /// The leaf type name used in graph-validation diagnostics.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn type_name(self) -> &'static str {
        self.type_name
    }

    /// Returns the leaf type identity, when the structure has one unambiguous
    /// value.
    ///
    /// # Returns
    ///
    /// `Some` with the leaf identity when the projection is unambiguous;
    /// otherwise `None`.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn identity(self) -> Option<TypeIdentity> {
        self.identity
    }
}

/// Projects a relation field through supported wrappers to its leaf type.
///
/// Optional, sequence, set, and array wrappers preserve a single leaf type;
/// map values deliberately do not because their relationship endpoint is
/// ambiguous.
///
/// # Parameters
///
/// - `field_type`: The field type to project through supported wrappers.
///
/// # Returns
///
/// The leaf projection used to compare relation endpoints.
#[must_use]
pub(crate) fn project_relation_type(field_type: TypeRef) -> RelationProjection {
    let mut current = field_type;
    loop {
        match current.shape() {
            TypeShape::Optional(inner) => {
                current = inner;
            }
            TypeShape::Sequence(inner) | TypeShape::Set(inner) => {
                current = inner;
            }
            TypeShape::Array { element, .. } => {
                current = element;
            }
            TypeShape::Map { .. } => {
                return RelationProjection::new(None, current.type_name());
            }
            TypeShape::Scalar(_) | TypeShape::Named(_) | TypeShape::Opaque => {
                return RelationProjection::new(Some(current.identity()), current.identity().type_name());
            }
        }
    }
}
