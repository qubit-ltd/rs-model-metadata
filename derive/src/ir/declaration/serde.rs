// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serde overlay IR.

// qubit-style: allow public-type-layout

use syn::LitStr;

#[derive(Clone, Default)]
pub(crate) struct SerdeIr {
    /// Explicit serialized field name.
    pub(crate) serialize_name: Option<LitStr>,
    /// Explicit deserialized field name.
    pub(crate) deserialize_name: Option<LitStr>,
    /// Whether serialization is skipped.
    pub(crate) skip_serializing: bool,
    /// Whether deserialization is skipped.
    pub(crate) skip_deserializing: bool,
    /// Whether the field is flattened.
    pub(crate) flatten: bool,
    /// Custom Serde adapter name.
    pub(crate) with: Option<LitStr>,
    /// Whether Serde uses a default.
    pub(crate) default: bool,
    /// Whether skip-serializing-if was explicitly set.
    pub(crate) explicit_skip_serializing_if: bool,
    /// Whether the model default supplies the Serde default.
    pub(crate) default_from_model: bool,
    /// Whether the field is omitted from the model surface.
    pub(crate) omit_from_model: bool,
    /// Whether omission was suppressed by an explicit declaration.
    pub(crate) omit_suppressed: bool,
}
