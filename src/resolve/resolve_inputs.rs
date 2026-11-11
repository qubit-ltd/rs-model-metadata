// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Inputs used by a single structure resolution attempt.

use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;

/// Supplies the registry snapshot and explicit roots for one resolution
/// attempt.
///
/// For a concrete type present in both the registry and `roots`, the explicit
/// root supplies the metadata used by this resolution. If roots contain the
/// same concrete type more than once, the last root is selected. Every root is
/// ABI-validated, including roots superseded by a later root. This selection
/// does not add entries or aliases to the registry.
///
/// # Type Parameters
///
/// - `'a` is the borrow lifetime of the registry and root slice, and also the
///   registry's metadata lifetime.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::ResolveInputs;
/// use qubit_model_metadata::resolve::StructureResolver;
///
/// // The process-wide registration flow provides the global model registry.
/// let models = ModelRegistry::try_global().expect("valid model registry");
/// let roots: &[&'static TypeMetadata] = &[];
/// let inputs = ResolveInputs { models, roots };
/// let graph = StructureResolver::for_roots(inputs)
///     .resolve()
///     .expect("empty roots resolve successfully");
/// assert!(graph.models().is_empty());
/// ```
#[derive(Clone, Copy)]
pub struct ResolveInputs<'a> {
    /// Registry containing concrete and generic model registrations.
    pub models: &'a ModelRegistry<'a>,
    /// Additional concrete roots, including anonymous and generic models.
    ///
    /// Roots take precedence over registry metadata for the same concrete
    /// type. The last root for a repeated concrete type is selected, but every
    /// root is ABI-validated. This does not modify the registry's indexes.
    pub roots: &'a [&'static TypeMetadata],
}
