// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;

/// A parsed value paired with the span of its literal or identifier.
pub(crate) struct SpannedValue<T> {
    /// The parsed semantic value.
    pub(crate) value: T,
    /// The span of the original value token.
    pub(crate) span: Span,
}
