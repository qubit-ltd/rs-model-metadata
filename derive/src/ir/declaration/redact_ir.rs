// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Normalized redaction policy IR.

use super::RedactModeIr;

/// Redaction policy normalized from one field's declaration attributes.
#[derive(Clone)]
pub(crate) struct RedactIr {
    /// Normalized redaction mode.
    pub(crate) mode: RedactModeIr,
}
