// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use syn::LitStr;

use super::model_attribute::ModelAttribute;

/// Parsed type-level model attributes split from ordinary model constraints.
pub(crate) struct ModelAttributes {
    /// Raw stable model-ID literals in source order.
    pub(crate) id: Vec<LitStr>,
    /// Parsed model constraints in source order.
    pub(crate) attributes: Vec<ModelAttribute>,
}
