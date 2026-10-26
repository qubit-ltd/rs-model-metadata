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
    pub roots: &'a [&'static TypeMetadata],
}
