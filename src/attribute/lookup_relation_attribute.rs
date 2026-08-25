// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;
use syn::TypePath;

use super::field_name::FieldName;

/// Parsed lookup-relation values.
pub(crate) struct LookupRelationAttribute {
    /// Target-model occurrences in source order.
    pub(crate) target: Vec<TypePath>,
    /// Target-field path occurrences in source order.
    pub(crate) target_field: Vec<Vec<FieldName>>,
    /// The span of the complete attribute item.
    pub(crate) span: Span,
}
