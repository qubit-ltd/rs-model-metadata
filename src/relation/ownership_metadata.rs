// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Metadata declaring model ownership.

use crate::type_metadata::NamedTypeRef;

/// Metadata declaring the named model that owns this model.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::NamedTypeRef;
/// use qubit_model_metadata::OwnershipMetadata;
/// use qubit_model_metadata::TypeIdentity;
///
/// let ownership = OwnershipMetadata::new(NamedTypeRef::unresolved(TypeIdentity::of::<u8>()));
/// assert!(ownership.owner().metadata().is_none());
/// ```
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
