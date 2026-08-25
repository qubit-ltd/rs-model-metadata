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

/// Parsed object-graph reference path segment.
pub(crate) enum ReferencePathSegment {
    /// Parent object navigation (`..`).
    Parent(Span),
    /// Named child field navigation.
    Field(FieldName),
}

/// Parsed direct-reference values.
pub(crate) struct ReferenceAttribute {
    /// Referenced entity occurrences in source order.
    pub(crate) entity: Vec<LitStr>,
    /// Referenced property path occurrences in source order.
    pub(crate) property: Vec<Vec<FieldName>>,
    /// Existing value occurrences in source order.
    pub(crate) existing: Vec<SpannedValue<bool>>,
    /// Object-graph reference path occurrences in source order.
    pub(crate) path: Vec<Vec<ReferencePathSegment>>,
    /// The span of the complete attribute item.
    pub(crate) span: Span,
}
