// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;

use super::spanned_value::SpannedValue;

/// Parsed sequence constraint values.
pub(crate) struct SequenceAttribute {
    /// Minimum item-count occurrences in source order.
    pub(crate) min_items: Vec<SpannedValue<u32>>,
    /// Maximum item-count occurrences in source order.
    pub(crate) max_items: Vec<SpannedValue<u32>>,
    /// Every `unique_items` marker span in source order.
    pub(crate) unique_items: Vec<Span>,
    /// The span of the complete attribute item.
    pub(crate) span: Span,
}
