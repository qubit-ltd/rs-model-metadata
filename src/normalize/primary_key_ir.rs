// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;

use super::primary_key_field_ir::PrimaryKeyFieldIr;
use crate::attribute::FieldName;

/// A canonical primary-key definition.
pub(crate) struct PrimaryKeyIr {
    /// Key fields in declaration order.
    pub(crate) fields: Vec<PrimaryKeyFieldIr>,
    /// Every `generated(...)` field reference in source order, including
    /// invalid or duplicate ones.
    pub(crate) generated: Vec<FieldName>,
    /// The originating attribute span.
    pub(crate) span: Span,
}
