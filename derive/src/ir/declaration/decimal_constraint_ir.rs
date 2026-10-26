// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Normalized decimal constraint values.

use syn::LitStr;

/// Normalized decimal constraint values, including lexical bounds and rounding
/// policy.
#[derive(Clone)]
pub(crate) struct DecimalConstraintIr {
    /// Significant-digit precision.
    pub(crate) precision: Option<u16>,
    /// Decimal scale.
    pub(crate) scale: u16,
    /// Rounding mode name.
    pub(crate) rounding: String,
    /// Whether the semantic is money.
    pub(crate) money: bool,
    /// Lower bound literal.
    pub(crate) min: Option<LitStr>,
    /// Upper bound literal.
    pub(crate) max: Option<LitStr>,
    /// Whether the lower bound is inclusive.
    pub(crate) min_inclusive: bool,
    /// Whether the upper bound is inclusive.
    pub(crate) max_inclusive: bool,
}
