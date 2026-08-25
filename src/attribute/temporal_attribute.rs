// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;

use super::spanned_value::SpannedValue;
use super::temporal_precision::TemporalPrecision;

/// Parsed temporal constraint values.
pub(crate) struct TemporalAttribute {
    /// Time-precision occurrences in source order.
    pub(crate) precision: Vec<SpannedValue<TemporalPrecision>>,
    /// The span of the complete attribute item.
    pub(crate) span: Span,
}
