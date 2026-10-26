// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Narrow declaration modes delegated to `qubit-redact` capabilities.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::RedactModeMetadata;
///
/// let mode = RedactModeMetadata::KeyedBy("account");
/// assert!(matches!(mode, RedactModeMetadata::KeyedBy("account")));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RedactModeMetadata {
    /// Apply the configured sensitivity level.
    Level,
    /// Omit the selected value.
    Skip,
    /// Recurse into nested metadata.
    Nested,
    /// Apply map-specific redaction.
    Map,
    /// Select a redaction policy by stable key.
    KeyedBy(&'static str),
    /// Use JSON redaction semantics.
    Json,
}
