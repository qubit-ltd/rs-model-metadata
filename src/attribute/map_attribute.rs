// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;

use super::spanned_value::SpannedValue;

/// Parsed map constraint values.
pub(crate) struct MapAttribute {
    /// Minimum entry-count occurrences in source order.
    pub(crate) min_entries: Vec<SpannedValue<u32>>,
    /// Maximum entry-count occurrences in source order.
    pub(crate) max_entries: Vec<SpannedValue<u32>>,
    /// The span of the complete attribute item.
    pub(crate) span: Span,
}
