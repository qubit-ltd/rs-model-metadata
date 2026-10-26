// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Normalized text constraint values.

/// Normalized text constraint values collected from declaration attributes.
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
