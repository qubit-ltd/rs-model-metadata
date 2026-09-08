// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Redaction IR.

// qubit-style: allow multiple-public-types

#[derive(Clone)]
pub(crate) struct RedactIr {
    /// Normalized redaction mode.
    pub(crate) mode: RedactModeIr,
}

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
