// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::decimal_semantic::DecimalSemantic;
use crate::attribute::DecimalAttribute;

/// Canonical decimal semantics shared by `decimal` and `money` syntax.
pub(crate) struct DecimalIr {
    /// Parsed decimal values.
    pub(crate) value: DecimalAttribute,
    /// Whether the value is an ordinary number or money.
    pub(crate) semantic: DecimalSemantic,
}
