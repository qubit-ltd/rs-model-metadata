// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Rounding policies for decimal constraints.

/// A rounding strategy for decimal constraints.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::RoundingMode;
///
/// assert_ne!(RoundingMode::HalfEven, RoundingMode::Down);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundingMode {
    /// Round toward zero.
    Down,
    /// Round away from zero.
    Up,
    /// Round toward positive infinity.
    Ceiling,
    /// Round toward negative infinity.
    Floor,
    /// Round to the nearest value, with halves rounded away from zero.
    HalfUp,
    /// Round to nearest, with halves toward zero.
    HalfDown,
    /// Round to the nearest value, with halves rounded to even.
    HalfEven,
    /// Reject values that require rounding.
    Unnecessary,
}
