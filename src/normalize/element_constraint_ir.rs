// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::decimal_ir::DecimalIr;
use crate::attribute::TextAttribute;

/// A normalized constraint supported on migrated collection elements.
pub(crate) enum ElementConstraintIr {
    /// Text constraints for string elements.
    Text(
        /// Parsed text-constraint values.
        TextAttribute,
    ),
    /// Ordinary decimal constraints for high-precision numeric elements.
    Decimal(
        /// Parsed decimal constraint values, or normalized decimal IR.
        DecimalIr,
    ),
}
