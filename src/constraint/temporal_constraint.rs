// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::TemporalPrecision;

/// Constraints for temporal values.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::TemporalConstraint;
/// use qubit_model_metadata::TemporalPrecision;
///
/// let constraint = TemporalConstraint::new(TemporalPrecision::Millisecond);
/// assert_eq!(constraint.precision(), TemporalPrecision::Millisecond);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemporalConstraint {
    /// The required temporal precision.
    precision: TemporalPrecision,
}

impl TemporalConstraint {
    /// Creates temporal constraints from precision and normalization semantics.
    ///
    /// # Parameters
    ///
    /// * `precision` - The required temporal precision.
    /// # Returns
    ///
    /// Temporal constraints containing the supplied precision.
    #[must_use]
    #[inline(always)]
    pub const fn new(precision: TemporalPrecision) -> Self {
        Self {
            precision,
        }
    }

    /// Returns the required temporal precision.
    ///
    /// # Returns
    ///
    /// The required temporal precision.
    #[must_use]
    #[inline(always)]
    pub const fn precision(self) -> TemporalPrecision {
        self.precision
    }
}
