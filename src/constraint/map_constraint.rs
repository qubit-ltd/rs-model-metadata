// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Constraints that apply to a map.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::MapConstraint;
///
/// let constraint = MapConstraint::new(Some(1), Some(8));
/// assert_eq!(constraint.min_entries(), Some(1));
/// assert_eq!(constraint.max_entries(), Some(8));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapConstraint {
    /// The minimum number of entries, if constrained.
    min_entries: Option<u32>,
    /// The maximum number of entries, if constrained.
    max_entries: Option<u32>,
}

impl MapConstraint {
    /// Creates map constraints from entry limits.
    ///
    /// # Parameters
    ///
    /// * `min_entries` - The optional minimum entry count.
    /// * `max_entries` - The optional maximum entry count.
    ///
    /// # Returns
    ///
    /// Map constraints containing the supplied entry limits.
    ///
    /// # Panics
    ///
    /// Panics when the supplied minimum entry count exceeds the maximum.
    #[must_use]
    pub const fn new(min_entries: Option<u32>, max_entries: Option<u32>) -> Self {
        if let (Some(min_entries), Some(max_entries)) = (min_entries, max_entries) {
            assert!(
                min_entries <= max_entries,
                "minimum entry count cannot exceed maximum entry count"
            );
        }
        Self {
            min_entries,
            max_entries,
        }
    }

    /// Returns the minimum number of entries, if constrained.
    ///
    /// # Returns
    ///
    /// `Some` with the minimum entry count when constrained; otherwise, `None`.
    #[must_use]
    #[inline(always)]
    pub const fn min_entries(self) -> Option<u32> {
        self.min_entries
    }

    /// Returns the maximum number of entries, if constrained.
    ///
    /// # Returns
    ///
    /// `Some` with the maximum entry count when constrained; otherwise, `None`.
    #[must_use]
    #[inline(always)]
    pub const fn max_entries(self) -> Option<u32> {
        self.max_entries
    }
}
