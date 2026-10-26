// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Normalized model-field declaration metadata.

use syn::Ident;
use syn::Type;
use syn::Visibility;

use super::super::super::Located;
use super::FieldOccurrence;

/// Normalized field declaration metadata consumed by model expansion.
#[derive(Clone)]
pub(crate) struct FieldIr {
    /// Enum variant containing this field, when applicable.
    pub(crate) variant_index: Option<usize>,
    /// Reflection declaration index.
    pub(crate) index: Located<usize>,
    /// Declared Rust field type.
    pub(crate) ty: Type,
    /// Normalized field attribute occurrences.
    pub(crate) occurrences: Vec<FieldOccurrence>,
    /// Whether Serde serialization remains enabled.
    pub(crate) keep_serializing: bool,
    /// Whether the source field has a name.
    pub(crate) named: bool,
    /// Source field identifier, absent for tuple fields.
    pub(crate) rust_name: Option<Ident>,
    /// Rust visibility written on the field.
    pub(crate) visibility: Visibility,
}
