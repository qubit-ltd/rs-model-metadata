// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;
use syn::LitStr;

use super::field_name::FieldName;

/// Parsed model attribute containing logical-name occurrences and ordered
/// fields.
pub(crate) struct NamedFieldsAttribute {
    /// Logical-name occurrences in source order.
    pub(crate) name: Vec<LitStr>,
    /// Fields in declaration order.
    pub(crate) fields: Vec<FieldName>,
    /// The span of the complete attribute item.
    pub(crate) span: Span,
}
