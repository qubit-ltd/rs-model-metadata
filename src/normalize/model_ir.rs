// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use syn::Ident;
use syn::LitStr;

use super::model_attribute_ir::ModelAttributeIr;
use super::model_shape_ir::ModelShapeIr;

/// An expansion-ready model with all shorthand syntax removed.
pub(crate) struct ModelIr {
    /// The declared model type name.
    pub(crate) ident: Ident,
    /// Raw stable model-ID literals in source order.
    pub(crate) id: Vec<LitStr>,
    /// Canonical model-level attributes.
    pub(crate) attributes: Vec<ModelAttributeIr>,
    /// Number of attributes declared directly on the model before field
    /// shorthands were appended.
    pub(crate) model_attribute_count: usize,
    /// Whether this named model is a textual value object.
    pub(crate) textual: bool,
    /// The model's supported structural form.
    pub(crate) shape: ModelShapeIr,
}
