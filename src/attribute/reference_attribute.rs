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
use super::spanned_value::SpannedValue;

/// Parsed direct-reference values.
pub(crate) struct ReferenceAttribute {
    /// Target-model occurrences in source order.
    pub(crate) target: Vec<LitStr>,
    /// Target-field path occurrences in source order.
    pub(crate) target_field: Vec<Vec<FieldName>>,
    /// Must-exist value occurrences in source order.
    pub(crate) must_exist: Vec<SpannedValue<bool>>,
    /// Same-as field-path occurrences in source order.
    pub(crate) same_as: Vec<Vec<FieldName>>,
    /// The span of the complete attribute item.
    pub(crate) span: Span,
}
