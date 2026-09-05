// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private intermediate representations for model implementation expansion.

mod getter_ir;
mod getter_return;
mod property_method;
mod setter_ir;

pub(super) use getter_ir::GetterIr;
pub(super) use getter_return::GetterReturn;
pub(super) use property_method::PropertyMethod;
pub(super) use setter_ir::SetterIr;
