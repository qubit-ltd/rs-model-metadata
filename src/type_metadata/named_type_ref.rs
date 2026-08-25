// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Static references to named model types.

use crate::type_metadata::HasTypeMetadata;
use crate::type_metadata::TypeIdentity;
use crate::type_metadata::TypeMetadata;
use crate::type_metadata::metadata_of;

/// A static reference to metadata for a named model type.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::NamedTypeRef;
/// use qubit_model_metadata::TypeIdentity;
///
/// let named = NamedTypeRef::unresolved(TypeIdentity::of::<u8>());
/// assert_eq!(named.identity(), TypeIdentity::of::<u8>());
/// assert!(named.metadata().is_none());
/// ```
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
    pub const fn new(identity: TypeIdentity, metadata: fn() -> &'static TypeMetadata) -> Self {
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
    #[must_use]
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
