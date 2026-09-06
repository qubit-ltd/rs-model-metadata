//! Inputs for constructing an isolated validation plan.

use crate::ResolvedModelGraph;
use qubit_validator::next::ValidatorRegistry;

/// Immutable registries used by one validation-plan build.
pub struct ValidationBuildInputs<'a> {
    /// The structure-only model graph to which declarations belong.
    pub graph: &'a ResolvedModelGraph<'a>,
    /// The local executable validator registry.
    pub validators: &'a ValidatorRegistry,
}
