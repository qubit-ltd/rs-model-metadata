// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Supported temporal precision levels.

/// The resolution retained for temporal values.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::TemporalPrecision;
///
/// assert_ne!(TemporalPrecision::Nanosecond, TemporalPrecision::Second);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporalPrecision {
    /// Whole seconds.
    Second,
    /// Milliseconds.
    Millisecond,
    /// Microseconds.
    Microsecond,
    /// Nanoseconds.
    Nanosecond,
}
