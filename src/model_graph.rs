// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Explicit validation of direct-reference graphs in a model registry.

mod model_graph_error;
mod model_graph_errors;
pub(crate) mod relation_projection;

pub use self::model_graph_error::ModelGraphError;
pub use self::model_graph_errors::ModelGraphErrors;
