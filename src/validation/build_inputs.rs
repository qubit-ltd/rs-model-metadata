//! Inputs for constructing an isolated validation plan.

// qubit-style: allow type-file-name

use qubit_validator::ValidatorRegistry;

use crate::ResolvedModelGraph;

/// Immutable registries used by one validation-plan build.
pub struct ValidationBuildInputs<'a> {
    /// The structure-only model graph to which declarations belong.
    pub graph: &'a ResolvedModelGraph<'a>,
    /// The local executable validator registry.
    pub validators: &'a ValidatorRegistry,
}
