// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Associates a parsed value with its source span.

use proc_macro2::Span;

/// Stores one parsed value and the source location that declared it.
#[derive(Clone)]
pub(crate) struct Located<T> {
    value: T,
    span: Span,
}

impl<T> Located<T> {
    /// Creates a located value from its parsed representation and source span.
    pub(crate) const fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }

    /// Returns the parsed value without changing its source location.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn value(&self) -> &T {
        &self.value
    }

    /// Returns the source span that declared this value.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn span(&self) -> Span {
        self.span
    }

}
