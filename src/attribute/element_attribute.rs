// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;

use super::element_constraint_attribute::ElementConstraintAttribute;

/// Parsed constraints applied to sequence elements.
pub(crate) struct ElementAttribute {
    /// Element constraints in source order.
    pub(crate) attributes: Vec<ElementConstraintAttribute>,
    /// The span of the complete attribute item.
    pub(crate) span: Span,
}
