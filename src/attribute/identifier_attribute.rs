// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;

/// Parsed identifier shorthand.
pub(crate) struct IdentifierAttribute {
    /// Every `generated` marker span in source order.
    pub(crate) generated: Vec<Span>,
    /// The span of the complete attribute item.
    pub(crate) span: Span,
}
