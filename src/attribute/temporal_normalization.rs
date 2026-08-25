// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Parsed temporal normalization policy.
#[derive(Clone, Copy)]
pub(crate) enum TemporalNormalization {
    /// Preserve the supplied representation.
    Preserve,
    /// Normalize to UTC.
    Utc,
}
