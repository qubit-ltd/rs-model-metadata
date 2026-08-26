// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::model_field::ModelField;

/// A parsed enum variant shape.
pub(crate) enum ModelVariantShape {
    /// A variant without payload fields.
    Unit,
    /// A variant with positional payload fields.
    Tuple(
        /// Positional fields in declaration order.
        Vec<ModelField>,
    ),
    /// A variant with named payload fields.
    Struct(
        /// Named fields in declaration order.
        Vec<ModelField>,
    ),
}
