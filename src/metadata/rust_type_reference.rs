// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::any::TypeId;
use core::any::type_name;
use core::fmt;
use core::hash::Hash;
use core::hash::Hasher;

/// A declaration-time reference to a Rust type.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::RustTypeReference;
///
/// let reference = RustTypeReference::of::<String>();
/// assert_eq!(reference.type_id(), core::any::TypeId::of::<String>());
/// assert!(reference.type_name().contains("String"));
/// ```
#[derive(Clone, Copy)]
pub struct RustTypeReference {
    /// Monomorphized identity query; no reflection descriptor is constructed.
    type_id: fn() -> TypeId,
    /// Monomorphized diagnostic name query for the same Rust type.
    type_name: fn() -> &'static str,
}

impl RustTypeReference {
    /// Creates a reference to `T` without constructing a runtime descriptor.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The `'static` Rust type whose identity and diagnostic name are
    ///   recorded.
    ///
    /// # Returns
    ///
    /// A compact reference that can later report `T`'s type identity and name.
    #[must_use]
    #[inline]
    pub const fn of<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>,
            type_name: type_name::<T>,
        }
    }

    /// Returns the referenced Rust [`TypeId`].
    ///
    /// # Returns
    ///
    /// The monomorphized `TypeId` for the referenced Rust type.
    #[must_use]
    #[inline]
    pub fn type_id(self) -> TypeId {
        (self.type_id)()
    }

    /// Returns the referenced Rust type name.
    ///
    /// # Returns
    ///
    /// The diagnostic name reported for the referenced Rust type.
    #[must_use]
    #[inline]
    pub fn type_name(self) -> &'static str {
        (self.type_name)()
    }
}

impl fmt::Debug for RustTypeReference {
    /// Formats the reference as a tuple containing its diagnostic type name.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("RustTypeReference")
            .field(&self.type_name())
            .finish()
    }
}

impl PartialEq for RustTypeReference {
    /// Compares references by the identity of their represented Rust types.
    fn eq(&self, other: &Self) -> bool {
        self.type_id() == other.type_id()
    }
}

impl Eq for RustTypeReference {}

impl Hash for RustTypeReference {
    /// Hashes the represented Rust type identity, matching [`PartialEq`].
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_id().hash(state);
    }
}
