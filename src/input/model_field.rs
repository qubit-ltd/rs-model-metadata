// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use syn::Type;

use crate::attribute::FieldAttribute;

/// A declared model field.
pub(crate) struct ModelField {
    /// The zero-based declaration ordinal.
    pub(crate) ordinal: usize,
    /// The normalized field name.
    pub(crate) name: String,
    /// The declared Rust field type.
    pub(crate) ty: Type,
    /// Parsed field-level attributes in source order.
    pub(crate) attributes: Vec<FieldAttribute>,
}
