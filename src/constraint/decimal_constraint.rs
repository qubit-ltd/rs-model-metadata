// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Precision, range, and rounding policies for decimal values.

use super::DecimalSemantic;
use super::RoundingMode;

/// Constraints for decimal values.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::DecimalConstraint;
/// use qubit_model_metadata::DecimalSemantic;
/// use qubit_model_metadata::RoundingMode;
///
/// let constraint = DecimalConstraint::new(Some(12), 2, RoundingMode::HalfEven, DecimalSemantic::Money);
/// assert_eq!(constraint.scale(), 2);
/// assert_eq!(constraint.semantic(), DecimalSemantic::Money);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecimalConstraint {
    /// The total significant-digit precision, if constrained.
    precision: Option<u16>,
    /// The number of decimal places.
    scale: u16,
    /// The required rounding strategy.
    rounding: RoundingMode,
    /// The domain meaning of the decimal value.
    semantic: DecimalSemantic,
    /// Exact lower bound as declared by the model.
    min: Option<&'static str>,
    /// Exact upper bound as declared by the model.
    max: Option<&'static str>,
    /// Whether the lower bound includes equality.
    min_inclusive: bool,
    /// Whether the upper bound includes equality.
    max_inclusive: bool,
}

impl DecimalConstraint {
    /// Creates decimal constraints from precision, scale, rounding, and
    /// semantic meaning.
    ///
    /// # Parameters
    ///
    /// * `precision` - The optional total significant-digit precision.
    /// * `scale` - The number of decimal places.
    /// * `rounding` - The required rounding strategy.
    /// * `semantic` - The domain meaning of the decimal value.
    ///
    /// # Returns
    ///
    /// Decimal constraints containing the supplied precision, scale, and
    /// policies.
    ///
    /// # Panics
    ///
    /// Panics when `scale` exceeds a supplied `precision`.
    #[must_use]
    pub const fn new(precision: Option<u16>, scale: u16, rounding: RoundingMode, semantic: DecimalSemantic) -> Self {
        if let Some(precision) = precision {
            assert!(scale <= precision, "decimal scale cannot exceed precision");
        }
        Self {
            precision,
            scale,
            rounding,
            semantic,
            min: None,
            max: None,
            min_inclusive: true,
            max_inclusive: true,
        }
    }

    /// Attaches exact declaration-time bounds.
    #[must_use]
    pub const fn with_bounds(
        mut self,
        min: Option<&'static str>,
        max: Option<&'static str>,
        min_inclusive: bool,
        max_inclusive: bool,
    ) -> Self {
        self.min = min;
        self.max = max;
        self.min_inclusive = min_inclusive;
        self.max_inclusive = max_inclusive;
        self
    }

    /// Returns the total significant-digit precision, if constrained.
    ///
    /// # Returns
    ///
    /// `Some` with the precision when constrained; otherwise, `None`.
    #[must_use]
    #[inline(always)]
    pub const fn precision(self) -> Option<u16> {
        self.precision
    }

    /// Returns the number of decimal places.
    ///
    /// # Returns
    ///
    /// The number of decimal places.
    #[must_use]
    #[inline(always)]
    pub const fn scale(self) -> u16 {
        self.scale
    }

    /// Returns the required rounding strategy.
    ///
    /// # Returns
    ///
    /// The required rounding strategy.
    #[must_use]
    #[inline(always)]
    pub const fn rounding(self) -> RoundingMode {
        self.rounding
    }

    /// Returns whether the value is an ordinary number or money.
    ///
    /// # Returns
    ///
    /// The domain meaning of the decimal value.
    #[must_use]
    #[inline(always)]
    pub const fn semantic(self) -> DecimalSemantic {
        self.semantic
    }

    /// Returns the exact lower-bound declaration.
    #[must_use]
    #[inline(always)]
    pub const fn min(self) -> Option<&'static str> {
        self.min
    }

    /// Returns the exact upper-bound declaration.
    #[must_use]
    #[inline(always)]
    pub const fn max(self) -> Option<&'static str> {
        self.max
    }

    /// Returns whether the lower bound includes equality.
    #[must_use]
    #[inline(always)]
    pub const fn min_inclusive(self) -> bool {
        self.min_inclusive
    }

    /// Returns whether the upper bound includes equality.
    #[must_use]
    #[inline(always)]
    pub const fn max_inclusive(self) -> bool {
        self.max_inclusive
    }
}
