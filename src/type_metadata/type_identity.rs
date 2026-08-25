// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Runtime identities for Rust types.

use core::any::TypeId;
use core::any::type_name;

/// A runtime identity for a Rust type, with its fully qualified name retained
/// for display.
///
/// This identity is local to the Rust process/build that produced it. It is
/// suitable for in-memory metadata lookup, but must not be persisted or used
/// as a stable cross-process identifier.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::TypeIdentity;
///
/// let identity = TypeIdentity::of::<String>();
/// assert_eq!(identity, TypeIdentity::of::<String>());
/// assert_ne!(identity, TypeIdentity::of::<u8>());
/// ```
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
    #[must_use]
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
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.debug_tuple("TypeIdentity").field(&self.type_name()).finish()
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
