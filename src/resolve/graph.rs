// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable structure graph and resolved structural views.

mod context_requirement;
mod model_graph;
mod projection_execution_error;
mod query_declaration;
mod query_metadata;
mod resolved_dependency;
mod resolved_projection_producer;
mod resolved_projection_source;
mod resolved_reference;

pub use context_requirement::ContextRequirement;
pub use model_graph::ModelGraph;
pub use projection_execution_error::ProjectionExecutionError;
pub use query_declaration::QueryDeclaration;
pub use query_metadata::QueryMetadata;
pub use resolved_dependency::ResolvedDependency;
pub use resolved_projection_producer::ResolvedProjectionProducer;
pub use resolved_projection_source::ResolvedProjectionSource;
pub use resolved_reference::ResolvedReference;
