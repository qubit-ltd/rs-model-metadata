// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Normalized constraint forms retained while compiling a model declaration.

use super::DecimalConstraintIr;
use super::TextConstraintIr;

/// Normalized constraint forms retained while compiling a model declaration.
#[derive(Clone)]
pub(crate) enum ConstraintIr {
    /// Character, byte-length, blankness, and semantic-format restrictions.
    Text(TextConstraintIr),
    /// Precision, scale, rounding, and bound restrictions for decimal values.
    Decimal(DecimalConstraintIr),
    /// Name of the temporal precision constraint applied to a field.
    Time(String),
    /// Minimum, maximum, and uniqueness requirements for a sequence.
    Sequence {
        /// Minimum item count.
        min: Option<usize>,
        /// Maximum item count.
        max: Option<usize>,
        /// Whether items must be unique.
        unique: bool,
    },
    /// Minimum and maximum entry-count requirements for a map.
    Map {
        /// Minimum entry count.
        min: Option<usize>,
        /// Maximum entry count.
        max: Option<usize>,
    },
}
