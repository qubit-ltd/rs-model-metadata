// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;

use super::element_constraint_ir::ElementConstraintIr;

/// Canonical constraints applied to sequence elements.
pub(crate) struct ElementIr {
    /// Element constraints in source order.
    pub(crate) attributes: Vec<ElementConstraintIr>,
    /// The originating attribute span.
    pub(crate) span: Span,
}
