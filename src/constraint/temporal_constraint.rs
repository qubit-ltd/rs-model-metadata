// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::TemporalNormalization;
use super::TemporalPrecision;

/// Constraints for temporal values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemporalConstraint {
    /// The required temporal precision.
    precision: TemporalPrecision,
    /// The temporal normalization policy.
    normalization: TemporalNormalization,
}

impl TemporalConstraint {
    /// Creates temporal constraints from precision and normalization semantics.
    ///
    /// # Parameters
    ///
    /// * `precision` - The required temporal precision.
    /// * `normalization` - The temporal normalization policy.
    ///
    /// # Returns
    ///
    /// Temporal constraints containing the supplied precision and policy.
    #[must_use]
    #[inline(always)]
    pub const fn new(precision: TemporalPrecision, normalization: TemporalNormalization) -> Self {
        Self {
            precision,
            normalization,
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

    /// Returns the temporal normalization policy.
    ///
    /// # Returns
    ///
    /// The temporal normalization policy.
    #[must_use]
    #[inline(always)]
    pub const fn normalization(self) -> TemporalNormalization {
        self.normalization
    }
}
