// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Selector and validator IR.

#[path = "selector_ir.rs"]
mod selector_ir;
#[path = "selector_position_ir.rs"]
mod selector_position_ir;
#[path = "validator_ir.rs"]
mod validator_ir;

pub(crate) use selector_ir::SelectorIr;
pub(crate) use selector_position_ir::SelectorPositionIr;
pub(crate) use validator_ir::ValidatorIr;
