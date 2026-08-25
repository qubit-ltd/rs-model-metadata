// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! A field name together with the source span that declared it.

use proc_macro2::Span;

/// A field name together with the source span that declared it.
pub(crate) struct FieldName {
    /// The normalized Rust field name.
    pub(crate) name: String,
    /// The span of the name in the attribute.
    pub(crate) span: Span,
}
