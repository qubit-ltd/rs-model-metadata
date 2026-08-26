// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::model_field::ModelField;
use super::model_variant::ModelVariant;

/// A supported structural form of a model declaration.
pub(crate) enum ModelShape {
    /// A struct with named fields in declaration order.
    NamedStruct(
        /// Named fields in declaration order.
        Vec<ModelField>,
    ),
    /// A struct with no fields.
    UnitStruct,
    /// A tuple struct with exactly one field.
    Newtype(
        /// The single tuple-struct field.
        Box<ModelField>,
    ),
    /// An enum with variants in declaration order.
    Enum(
        /// Parsed variants in declaration order.
        Vec<ModelVariant>,
    ),
}
