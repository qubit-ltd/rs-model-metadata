// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;
use syn::LitStr;

/// Parsed external strategy name.
pub(crate) struct StrategyAttribute {
    /// Stable logical strategy-name occurrences in source order.
    pub(crate) name: Vec<LitStr>,
    /// The span of the complete attribute item.
    pub(crate) span: Span,
}
