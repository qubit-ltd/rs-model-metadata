// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Redaction IR.

/// Redaction policy normalized from one field's declaration attributes.
#[derive(Clone)]
pub(crate) struct RedactIr {
    /// Normalized redaction mode.
    pub(crate) mode: RedactModeIr,
}

/// Redaction operation applied when producing reflected or serialized values.
#[derive(Clone)]
pub(crate) enum RedactModeIr {
    /// Sensitivity level name.
    Level(String),
    /// Omit the value.
    Skip,
    /// Recurse into nested metadata.
    Nested,
    /// Apply map redaction.
    Map,
    /// Select a keyed policy.
    KeyedBy(String),
    /// Apply JSON redaction.
    Json,
}
