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
mod owned_property_path;
mod queries;
mod reference_selection_ext;
mod relations;
mod resolver;
#[cfg(test)]
mod tests;

pub use error::ModelResolutionCause;
pub use error::ResolveError;
pub use error::ResolveErrorKind;
pub use error::ResolveErrors;
pub use graph::ModelGraph;
pub use graph::ProjectionExecutionError;
pub use graph::QueryField;
pub use graph::QueryMetadata;
pub use graph::ResolvedProjectionProducer;
pub use graph::ResolvedProjectionSource;
pub use graph::ResolvedReference;
pub use graph::UniqueQueryKey;
pub use resolver::ResolveInputs;
pub use resolver::StructureResolver;
