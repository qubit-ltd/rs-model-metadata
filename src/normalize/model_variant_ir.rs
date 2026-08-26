// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::model_variant_shape_ir::ModelVariantShapeIr;

/// An expansion-ready enum variant.
pub(crate) struct ModelVariantIr {
    /// The zero-based declaration ordinal.
    pub(crate) ordinal: usize,
    /// The normalized serialized variant name.
    pub(crate) name: String,
    /// The variant's normalized structural form.
    pub(crate) shape: ModelVariantShapeIr,
}
