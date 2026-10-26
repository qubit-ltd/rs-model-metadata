// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Redaction declaration IR types.

#[path = "redact_ir.rs"]
mod redact_ir;
#[path = "redact_mode_ir.rs"]
mod redact_mode_ir;

pub(crate) use redact_ir::RedactIr;
pub(crate) use redact_mode_ir::RedactModeIr;
