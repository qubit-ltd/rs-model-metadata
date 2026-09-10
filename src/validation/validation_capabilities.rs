// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structural validation execution capability checks.

use crate::metadata::TypeMetadata;
use crate::resolve::ModelGraph;
use crate::validation::ValidationBuildErrors;
use crate::validation::internal::declaration_walker;
use crate::validation::internal::occurrence_binder;

/// Checks the access shapes supported by the validation executor.
///
/// Success does not imply that custom rules are registered or an instance is
/// valid. This check does not bind a rule registry or invoke getters.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::ResolveInputs;
/// use qubit_model_metadata::resolve::StructureResolver;
/// use qubit_model_metadata::validation::ValidationCapabilities;
///
/// #[Model]
/// struct Profile { #[validator(id = "application.check_name")] name: String }
/// # fn main() {
/// let root = TypeMetadata::of::<Profile>();
/// let roots = [root];
/// let models = ModelRegistry::from_metadata(&[]).expect("isolated registry");
/// let graph = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
///     .resolve().expect("readable field structure");
/// // No custom rule registry is needed to check the access shape.
/// ValidationCapabilities::check(root, &graph).expect("supported shape");
/// # }
/// ```
pub enum ValidationCapabilities {
    // empty
}

impl ValidationCapabilities {
    /// Checks all executable declarations reachable from the supplied root.
    ///
    /// # Parameters
    ///
    /// - `root`: selects the concrete TypeId to check; declarations come from
    ///   the graph's canonical metadata, not an alternate caller overlay.
    /// - `graph`: already resolved graph containing the root and nested models.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when every execution declaration has a supported access
    /// shape, independently of custom rule registration and instance values.
    ///
    /// # Errors
    /// Returns unsupported declarations and access paths in source order, or
    /// `RootNotInGraph` when the supplied graph does not include the root.
    pub fn check(root: &'static TypeMetadata, graph: &ModelGraph<'_>) -> Result<(), ValidationBuildErrors> {
        let (occurrences, mut errors) = declaration_walker::collect(root, graph);
        for occurrence in occurrences {
            if let Err(error) = occurrence_binder::check_access(&occurrence, graph) {
                errors.push(*error);
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationBuildErrors::from_errors(errors))
        }
    }
}
