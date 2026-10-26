// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Resolved dependency occurrences retained by a structure graph.

use super::context_requirement::ContextRequirement;
use crate::metadata::DependencyBindingMetadata;

/// A dependency occurrence with its independently retained context requirement.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::ResolveInputs;
/// use qubit_model_metadata::resolve::ResolvedDependency;
/// use qubit_model_metadata::resolve::StructureResolver;
///
/// #[Model]
/// struct Draft { title: String }
/// # fn main() {
/// let models = ModelRegistry::try_global().expect("model registry");
/// let root = TypeMetadata::of::<Draft>();
/// let graph = StructureResolver::for_static_roots(&models, [root])
///     .resolve().expect("valid structure");
/// let dependencies: &[ResolvedDependency] = graph.dependencies();
/// assert!(dependencies.is_empty());
/// # }
/// ```
#[derive(Debug)]
pub struct ResolvedDependency {
    /// The exact dependency declaration retained from its validator occurrence.
    pub(in crate::resolve) declaration: &'static DependencyBindingMetadata,
    /// Whether resolving the object path requires a caller-supplied parent.
    pub(in crate::resolve) context: ContextRequirement,
}

impl ResolvedDependency {
    /// Returns the original occurrence, including source and separate paths.
    ///
    /// # Returns
    ///
    /// The static declaration retained for this dependency occurrence.
    #[must_use]
    #[inline]
    pub const fn declaration(&self) -> &'static DependencyBindingMetadata {
        self.declaration
    }

    /// Reports whether the consumer must supply a containing object.
    ///
    /// # Returns
    ///
    /// [`ContextRequirement::ParentObject`] when path resolution needs a
    /// caller-provided parent, or [`ContextRequirement::None`] otherwise.
    #[must_use]
    #[inline]
    pub const fn context_requirement(&self) -> ContextRequirement {
        self.context
    }
}
