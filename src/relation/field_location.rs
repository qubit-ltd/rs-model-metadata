// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Process-local identity of a concrete model field.

use std::any::TypeId;

/// Identifies one field of a concrete Rust type, independently of its address.
///
/// Enum variants have separate field index spaces. Generic instantiations have
/// distinct owners. This identity is only meaningful within the current
/// process; use a model ID for persistent or external identifiers.
///
/// # Examples
///
/// ```
/// use std::any::TypeId;
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// #[Model]
/// struct Record { name: String }
/// # fn main() {
/// let field = TypeMetadata::of::<Record>().field("name").unwrap();
/// let location = field.location().unwrap();
/// assert_eq!(location.owner(), TypeId::of::<Record>());
/// assert_eq!(location.variant(), None);
/// assert_eq!(location.index(), 0);
/// assert_eq!((*field).location(), Some(location));
/// # }
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FieldLocation {
    /// Concrete owner, including generic arguments.
    owner: TypeId,
    /// Variant index, absent for struct fields.
    variant: Option<usize>,
    /// Field index within its struct or variant.
    index: usize,
}

impl FieldLocation {
    /// Creates the identity supplied by the generated concrete field overlay.
    pub(crate) const fn new(owner: TypeId, variant: Option<usize>, index: usize) -> Self {
        Self { owner, variant, index }
    }

    /// Returns the concrete owner type identity.
    #[must_use]
    #[inline(always)]
    pub const fn owner(&self) -> TypeId {
        self.owner
    }

    /// Returns the variant index, or `None` for a struct field.
    #[must_use]
    #[inline(always)]
    pub const fn variant(&self) -> Option<usize> {
        self.variant
    }

    /// Returns the source field index within the struct or variant.
    #[must_use]
    #[inline(always)]
    pub const fn index(&self) -> usize {
        self.index
    }
}
