// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Inputs for constructing an isolated validation plan.

use std::sync::Arc;

use qubit_validator::ValidatorRegistry;

use crate::resolve::ModelGraph;

/// Shared graph and validator registry used for one validation-plan build.
///
/// The plan takes ownership of the graph handle. The validator registry is
/// borrowed only while binding rules and need not outlive the plan.
///
/// # Type Parameters
///
/// * `'registry` — lifetime of the model registry referenced by the graph.
/// * `'validators` — lifetime of the validator registry used during binding.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::ResolveInputs;
/// use qubit_model_metadata::resolve::StructureResolver;
/// use qubit_model_metadata::validation::ValidationBuildInputs;
/// use qubit_validator::ValidatorRegistry;
/// use std::sync::Arc;
///
/// #[Model]
/// struct Profile;
/// # fn main() {
/// let root = TypeMetadata::of::<Profile>();
/// let roots = [root];
/// let models = ModelRegistry::from_static_metadata(&[]).expect("isolated registry");
/// let graph = Arc::new(StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
///     .resolve().expect("valid structure"));
/// let validators = ValidatorRegistry::empty();
/// let inputs = ValidationBuildInputs { graph: Arc::clone(&graph), validators: &validators };
/// assert!(Arc::ptr_eq(&inputs.graph, &graph));
/// assert!(std::ptr::eq(inputs.validators, &validators));
/// # }
/// ```
pub struct ValidationBuildInputs<'registry, 'validators> {
    /// The structure-only model graph to which declarations belong.
    pub graph: Arc<ModelGraph<'registry>>,
    /// The local executable validator registry, borrowed during binding.
    pub validators: &'validators ValidatorRegistry,
}
