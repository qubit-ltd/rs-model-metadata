// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Resolution interface for static model metadata.

use crate::type_metadata::TypeIdentity;
use crate::type_metadata::TypeMetadata;

/// Resolves model metadata by its runtime type identity.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::MetadataResolver;
/// use qubit_model_metadata::ModelRegistry;
/// use qubit_model_metadata::TypeIdentity;
///
/// let registry = ModelRegistry::from_registrations([]).expect("empty registry is valid");
/// assert!(registry.resolve(TypeIdentity::of::<u8>()).is_none());
/// ```
pub trait MetadataResolver {
    /// Resolves `identity` to the first matching metadata entry.
    ///
    /// # Parameters
    ///
    /// * `identity` - The runtime identity to look up.
    ///
    /// # Returns
    ///
    /// `Some` with the first matching static metadata entry, or `None` when
    /// the identity is not present in the resolver.
    #[must_use]
    fn resolve(&self, identity: TypeIdentity) -> Option<&'static TypeMetadata>;
}
