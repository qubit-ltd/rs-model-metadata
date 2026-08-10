// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Comparison semantics for a field within a unique constraint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UniqueComparison {
    /// Compare values exactly.
    Exact,
    /// Compare text values without case sensitivity.
    IgnoreCase,
}
