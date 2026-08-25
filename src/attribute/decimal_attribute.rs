// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;

use super::rounding_mode::RoundingMode;
use super::spanned_value::SpannedValue;

/// Parsed decimal constraint values shared by `decimal` and `money`.
pub(crate) struct DecimalAttribute {
    /// Total significant-digit precision occurrences in source order.
    pub(crate) precision: Vec<SpannedValue<u16>>,
    /// Scale occurrences retained so later validation can require one for
    /// money.
    pub(crate) scale: Vec<SpannedValue<u16>>,
    /// Rounding-mode occurrences in source order.
    pub(crate) rounding: Vec<SpannedValue<RoundingMode>>,
    /// The span of the complete attribute item.
    pub(crate) span: Span,
}
