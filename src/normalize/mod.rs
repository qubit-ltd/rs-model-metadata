// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Normalization from parsed attribute syntax to expansion-ready semantic IR.

mod convert;
mod decimal_ir;
mod decimal_semantic;
mod element_constraint_ir;
mod element_ir;
mod field_attribute_ir;
mod field_ir;
mod model_attribute_ir;
mod model_ir;
mod model_shape_ir;
mod named_fields_ir;
mod ownership_ir;
mod primary_key_field_ir;
mod primary_key_ir;
mod unique_field_ir;
mod unique_ir;

pub(crate) use convert::normalize;
pub(crate) use decimal_ir::DecimalIr;
pub(crate) use decimal_semantic::DecimalSemantic;
pub(crate) use element_constraint_ir::ElementConstraintIr;
pub(crate) use element_ir::ElementIr;
pub(crate) use field_attribute_ir::FieldAttributeIr;
pub(crate) use field_ir::FieldIr;
pub(crate) use model_attribute_ir::ModelAttributeIr;
pub(crate) use model_ir::ModelIr;
pub(crate) use model_shape_ir::ModelShapeIr;
pub(crate) use named_fields_ir::NamedFieldsIr;
pub(crate) use ownership_ir::OwnershipIr;
pub(crate) use primary_key_field_ir::PrimaryKeyFieldIr;
pub(crate) use primary_key_ir::PrimaryKeyIr;
pub(crate) use unique_field_ir::UniqueFieldIr;
pub(crate) use unique_ir::UniqueIr;
