// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Normalized enum-variant declaration metadata.

use super::FieldIr;

/// Normalized enum variant names and its declared fields.
#[derive(Clone)]
pub(crate) struct VariantIr {
    /// Rust source variant name.
    pub(crate) rust_name: String,
    /// Canonical metadata variant name.
    pub(crate) canonical_name: String,
    /// Serialized variant name.
    pub(crate) serialized_name: String,
    /// Deserialized variant name.
    pub(crate) deserialized_name: String,
    /// Whether this is the default variant.
    pub(crate) default: bool,
    /// Normalized variant fields.
    pub(crate) fields: Vec<FieldIr>,
}
