// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Normalized validation constraints.

#[path = "constraint_ir.rs"]
mod constraint_ir;
#[path = "decimal_constraint_ir.rs"]
mod decimal_constraint_ir;
#[path = "text_constraint_ir.rs"]
mod text_constraint_ir;

pub(crate) use constraint_ir::ConstraintIr;
pub(crate) use decimal_constraint_ir::DecimalConstraintIr;
pub(crate) use text_constraint_ir::TextConstraintIr;
