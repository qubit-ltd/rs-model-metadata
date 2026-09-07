// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Explicit cross-model resolution and immutable resolved views.

mod runtime;

pub use runtime::ModelResolutionCause;
pub use runtime::ModelResolveError;
pub use runtime::ModelResolveErrorKind;
pub use runtime::ModelResolveErrors;
pub use runtime::ModelResolver;
pub use runtime::ProjectionExecutionError;
pub use runtime::QueryField;
pub use runtime::QueryMetadata;
pub use runtime::ResolveInputs;
pub use runtime::ResolvedCodec;
pub use runtime::ResolvedModelGraph;
pub use runtime::ResolvedProjectionProducer;
pub use runtime::ResolvedProjectionSource;
pub use runtime::ResolvedReference;
pub use runtime::UniqueQueryKey;
