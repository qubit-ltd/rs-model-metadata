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

/// Parsed single-field unique shorthand.
pub(crate) struct FieldUniqueAttribute {
    /// Optional logical constraint name.
    pub(crate) name: Vec<LitStr>,
    /// Fields whose values scope the current field's uniqueness.
    pub(crate) respect_to: Vec<FieldName>,
    /// Explicit Java-compatible ignore-case values.
    pub(crate) ignore_case_values: Vec<SpannedValue<bool>>,
    /// Whether the legacy `ignore_case` marker was used.
    pub(crate) legacy_ignore_case: bool,
    /// The span of the complete attribute item.
    pub(crate) span: Span,
}
