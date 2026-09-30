// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Inputs for constructing an isolated validation plan.

// qubit-style: allow type-file-name

use qubit_validator::ValidatorRegistry;

use crate::resolve::ModelGraph;

/// Immutable registries borrowed for one validation-plan build.
///
/// Both references share `'a`, so the graph and validator registry must remain
/// available for the same lifetime as these inputs. This value borrows them and
/// does not take ownership or clone either registry.
///
/// # Type Parameters
///
/// * `'a` — lifetime shared by the borrowed graph and validator registry.
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
///
/// #[Model]
/// struct Profile;
/// # fn main() {
/// let root = TypeMetadata::of::<Profile>();
/// let roots = [root];
/// let models = ModelRegistry::from_static_metadata(&[]).expect("isolated registry");
/// let graph = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
///     .resolve().expect("valid structure");
/// let validators = ValidatorRegistry::empty();
/// let inputs = ValidationBuildInputs { graph: &graph, validators: &validators };
/// assert!(std::ptr::eq(inputs.graph, &graph));
/// assert!(std::ptr::eq(inputs.validators, &validators));
/// # }
/// ```
pub struct ValidationBuildInputs<'a> {
    /// The structure-only model graph to which declarations belong, borrowed
    /// for `'a`.
    pub graph: &'a ModelGraph<'a>,
    /// The local executable validator registry, borrowed for `'a`.
    pub validators: &'a ValidatorRegistry,
}
