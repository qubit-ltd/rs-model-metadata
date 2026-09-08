// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Normalized validation constraints.

// qubit-style: allow multiple-public-types

use syn::LitStr;

#[derive(Clone)]
pub(crate) enum ConstraintIr {
    /// Text constraints.
    Text(TextConstraintIr),
    /// Decimal constraints.
    Decimal(DecimalConstraintIr),
    /// Temporal precision name.
    Time(String),
    Sequence {
        /// Minimum item count.
        min: Option<usize>,
        /// Maximum item count.
        max: Option<usize>,
        /// Whether items must be unique.
        unique: bool,
    },
    Map {
        /// Minimum entry count.
        min: Option<usize>,
        /// Maximum entry count.
        max: Option<usize>,
    },
}

#[derive(Clone, Default)]
pub(crate) struct TextConstraintIr {
    /// Minimum character count.
    pub(crate) min_chars: Option<u32>,
    /// Maximum character count.
    pub(crate) max_chars: Option<u32>,
    /// Minimum UTF-8 byte count.
    pub(crate) min_bytes: Option<u32>,
    /// Maximum UTF-8 byte count.
    pub(crate) max_bytes: Option<u32>,
    /// Character-set name.
    pub(crate) allowed_chars: Option<String>,
    /// Whether blank text is rejected.
    pub(crate) non_blank: bool,
    /// Semantic text format name.
    pub(crate) format: Option<String>,
}

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
