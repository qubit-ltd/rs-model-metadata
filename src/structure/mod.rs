// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shared, one-layer traversal of reflected structural relationships.

mod children;
mod internal;

pub(crate) use children::children;
pub(crate) use internal::StructuralEdge;
pub(crate) use internal::StructuralEdgeKind;
