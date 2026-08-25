// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;
use syn::LitStr;

/// A canonical named ordered-field declaration.
pub(crate) struct NamedFieldsIr {
    /// Logical-name occurrences in source order.
    pub(crate) name: Vec<LitStr>,
    /// Ordered normalized field names and their spans.
    pub(crate) fields: Vec<(String, Span)>,
    /// The originating attribute span.
    pub(crate) span: Span,
    /// Whether this index was generated from a reference attribute.
    pub(crate) implicit: bool,
}
