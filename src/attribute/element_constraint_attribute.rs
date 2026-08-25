// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::decimal_attribute::DecimalAttribute;
use super::text_attribute::TextAttribute;

/// A parsed constraint supported on migrated collection elements.
pub(crate) enum ElementConstraintAttribute {
    /// Text constraints for string elements.
    Text(
        /// Parsed text-constraint values.
        TextAttribute,
    ),
    /// Decimal constraints for high-precision numeric elements.
    Decimal(
        /// Parsed decimal constraint values, or normalized decimal IR.
        DecimalAttribute,
    ),
}
