// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;

use super::spanned_value::SpannedValue;
use super::text_format::TextFormat;
use super::allowed_chars::AllowedChars;

/// Parsed text constraint values.
pub(crate) struct TextAttribute {
    /// Minimum Unicode scalar count occurrences in source order.
    pub(crate) min_chars: Vec<SpannedValue<u32>>,
    /// Maximum Unicode scalar count occurrences in source order.
    pub(crate) max_chars: Vec<SpannedValue<u32>>,
    /// Minimum UTF-8 byte count occurrences in source order.
    pub(crate) min_bytes: Vec<SpannedValue<u32>>,
    /// Maximum UTF-8 byte count occurrences in source order.
    pub(crate) max_bytes: Vec<SpannedValue<u32>>,
    /// Allowed-character-set occurrences in source order.
    pub(crate) allowed_chars: Vec<SpannedValue<AllowedChars>>,
    /// Every `non_blank` marker span in source order.
    pub(crate) non_blank: Vec<Span>,
    /// Semantic text-format occurrences in source order.
    pub(crate) format: Vec<SpannedValue<TextFormat>>,
    /// The span of the complete attribute item.
    pub(crate) span: Span,
}
