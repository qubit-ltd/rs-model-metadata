// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::field_ir::FieldIr;
use crate::input::ModelVariant;

/// A supported model shape containing normalized fields.
pub(crate) enum ModelShapeIr {
    /// A struct with named fields in declaration order.
    NamedStruct(
        /// Named fields in declaration order.
        Vec<FieldIr>,
    ),
    /// A struct with no fields.
    UnitStruct,
    /// A tuple struct with exactly one field.
    Newtype(
        /// The single tuple-struct field.
        Box<FieldIr>,
    ),
    /// An enum whose variants all have no fields.
    FieldlessEnum(
        /// Fieldless variants in declaration order.
        Vec<ModelVariant>,
    ),
}
