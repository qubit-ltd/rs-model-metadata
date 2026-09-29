// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! A direct relationship to a reflected child type.

use qubit_reflect::descriptor::TypeRef;

use crate::structure::internal::StructuralEdgeKind;

/// A child type reference and its static structural position.
///
/// The target retains its resolved, opaque, or symbolic classification. The
/// position describes the declaration, rather than an executable property path.
#[derive(Clone, Copy, Debug)]
pub(crate) struct StructuralEdge {
    /// The original child relation, without requiring descriptor resolution.
    pub(crate) target: &'static TypeRef,
    /// The relationship and available source declaration position.
    /// Validation suffix rendering and internal contract tests read this fact.
    #[cfg_attr(
        not(any(feature = "validation", test)),
        allow(
            dead_code,
            reason = "structural positions are consumed by validation and internal contract tests"
        )
    )]
    pub(crate) kind: StructuralEdgeKind,
}
