// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// A rounding strategy for decimal constraints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundingMode {
    /// Round toward zero.
    Down,
    /// Round away from zero.
    Up,
    /// Round to the nearest value, with halves rounded away from zero.
    HalfUp,
    /// Round to the nearest value, with halves rounded to even.
    HalfEven,
}
