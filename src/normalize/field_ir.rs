// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;
use syn::Type;

use super::field_attribute_ir::FieldAttributeIr;

/// A field whose attributes have been normalized to runtime semantics.
pub(crate) struct FieldIr {
    /// The zero-based declaration ordinal.
    pub(crate) ordinal: usize,
    /// The normalized field name.
    pub(crate) name: String,
    /// The declared Rust type.
    pub(crate) ty: Type,
    /// Canonical field-level attributes.
    pub(crate) attributes: Vec<FieldAttributeIr>,
    /// Every marker span requiring the field type to be treated as opaque.
    pub(crate) opaque: Vec<Span>,
}
