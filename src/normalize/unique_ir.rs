// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;
use syn::LitStr;

use super::unique_field_ir::UniqueFieldIr;
use crate::attribute::FieldName;

/// A canonical unique-constraint definition.
pub(crate) struct UniqueIr {
    /// Logical-name occurrences in source order.
    pub(crate) name: Vec<LitStr>,
    /// Fields in comparison order.
    pub(crate) fields: Vec<UniqueFieldIr>,
    /// Every `ignore_case(...)` field reference, including invalid or
    /// duplicate ones.
    pub(crate) ignore_case: Vec<FieldName>,
    /// The originating attribute span.
    pub(crate) span: Span,
}
