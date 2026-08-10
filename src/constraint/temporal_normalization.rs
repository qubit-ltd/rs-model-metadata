// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// The normalization policy for temporal values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporalNormalization {
    /// Preserve the value's supplied offset or timezone representation.
    Preserve,
    /// Normalize to UTC.
    Utc,
}
