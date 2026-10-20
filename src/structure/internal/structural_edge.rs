// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! A direct relationship to a reflected child type.

use qubit_reflect::TypeDescriptor;

use crate::structure::internal::StructuralEdgeKind;

/// A child type reference and its static structural position.
///
/// The target retains its resolved, opaque, or symbolic classification. The
/// position describes the declaration, rather than an executable property path.
#[derive(Clone, Copy)]
pub(crate) struct StructuralEdge<'ctx> {
    /// A concrete child descriptor bound to the traversal context.
    pub(crate) target: TypeDescriptor<'ctx>,
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
