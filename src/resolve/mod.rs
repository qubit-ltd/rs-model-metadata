// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Explicit cross-model resolution and immutable resolved views.

mod error;
mod graph;
mod internal;
mod model_resolution_cause;
mod queries;
mod reference_selection_ext;
mod relations;
mod resolve_error;
mod resolve_error_kind;
mod resolve_errors;
mod resolve_inputs;
mod resolver;
mod structure_resolver;
#[cfg(test)]
mod tests;

pub use error::ModelResolutionCause;
pub use error::ResolveError;
pub use error::ResolveErrorKind;
pub use error::ResolveErrors;
pub use graph::ContextRequirement;
pub use graph::ModelGraph;
pub use graph::ProjectionExecutionError;
pub use graph::QueryDeclaration;
pub use graph::QueryMetadata;
pub use graph::ResolvedDependency;
pub use graph::ResolvedProjectionProducer;
pub use graph::ResolvedProjectionSource;
pub use graph::ResolvedReference;
pub use resolve_inputs::ResolveInputs;
pub use resolver::StructureResolver;
