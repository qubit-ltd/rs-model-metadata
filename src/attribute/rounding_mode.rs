// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Parsed decimal rounding mode.
#[derive(Clone, Copy)]
pub(crate) enum RoundingMode {
    /// Round toward zero.
    Down,
    /// Round away from zero.
    Up,
    /// Round halves away from zero.
    HalfUp,
    /// Round halves to even.
    HalfEven,
}
