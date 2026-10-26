// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! State retained only during one structural resolution attempt.

mod node_discovery;
mod owned_property_path;
mod resolution_context;
mod resolve_scope;

pub(super) use node_discovery::discover_nodes;
pub(super) use owned_property_path::OwnedPropertyPath;
pub(super) use resolution_context::ResolutionContext;
pub(super) use resolve_scope::ResolveScope;
