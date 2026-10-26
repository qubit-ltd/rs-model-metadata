// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Normalized redaction operation IR.

/// Redaction operation applied when producing reflected or serialized values.
#[derive(Clone)]
pub(crate) enum RedactModeIr {
    /// Sensitivity level name.
    Level(
        /// Sensitivity label used to select the redaction policy.
        String,
    ),
    /// Omit the value.
    Skip,
    /// Recurse into nested metadata.
    Nested,
    /// Apply map redaction.
    Map,
    /// Select a keyed policy.
    KeyedBy(
        /// Field path whose value selects the redaction policy.
        String,
    ),
    /// Apply JSON redaction.
    Json,
}
