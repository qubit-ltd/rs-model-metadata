// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Indexed query metadata retained during structure resolution.

use super::query_declaration::QueryDeclaration;

/// Direct indexed declarations available to downstream query generators.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::QueryMetadata;
/// use qubit_model_metadata::resolve::ResolveInputs;
/// use qubit_model_metadata::resolve::StructureResolver;
///
/// #[Model]
/// struct Draft { title: String }
/// # fn main() {
/// let models = ModelRegistry::try_global().expect("model registry");
/// let root = TypeMetadata::of::<Draft>();
/// let graph = StructureResolver::for_static_roots(&models, [root])
///     .resolve().expect("valid structure");
/// let query: Option<&QueryMetadata> = graph.query(root.type_id());
/// assert!(query.is_none());
/// # }
/// ```
#[derive(Debug)]
pub struct QueryMetadata {
    /// Direct declarations in source field order.
    pub(in crate::resolve) declarations: Box<[QueryDeclaration]>,
}

impl QueryMetadata {
    /// Returns declarations without choosing filter operators or external
    /// names.
    ///
    /// # Returns
    ///
    /// Indexed declarations in source field order. No operators or external
    /// query names are inferred.
    #[must_use]
    #[inline]
    pub fn declarations(&self) -> &[QueryDeclaration] {
        &self.declarations
    }
}
