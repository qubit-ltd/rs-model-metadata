// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Resolved source metadata for Projection types.

use crate::metadata::TypeMetadata;

/// A successfully resolved Projection source.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::ResolveInputs;
/// use qubit_model_metadata::resolve::ResolvedProjectionSource;
/// use qubit_model_metadata::resolve::StructureResolver;
///
/// #[Model]
/// struct Draft { title: String }
/// # fn main() {
/// let models = ModelRegistry::try_global().expect("model registry");
/// let root = TypeMetadata::of::<Draft>();
/// let graph = StructureResolver::for_static_roots(&models, [root])
///     .resolve().expect("valid structure");
/// let source: Option<&ResolvedProjectionSource> = graph.projection_source(root.type_id());
/// assert!(source.is_none());
/// # }
/// ```
#[derive(Debug)]
pub struct ResolvedProjectionSource {
    /// The resolved entity model supplying the projection.
    pub(in crate::resolve) target: &'static TypeMetadata,
}

impl ResolvedProjectionSource {
    /// Returns the resolved entity model supplying the projection.
    ///
    /// # Returns
    ///
    /// Static metadata for the Entity that supplies the Projection.
    #[must_use]
    #[inline]
    pub const fn target(&self) -> &'static TypeMetadata {
        self.target
    }
}
