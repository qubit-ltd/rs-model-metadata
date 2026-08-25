// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// The normalization policy for temporal values.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::TemporalNormalization;
///
/// assert_ne!(TemporalNormalization::Utc, TemporalNormalization::Preserve);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporalNormalization {
    /// Preserve the value's supplied offset or timezone representation.
    Preserve,
    /// Normalize to UTC.
    Utc,
}
