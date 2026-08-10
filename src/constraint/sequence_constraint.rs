// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Constraints that apply to an ordered sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SequenceConstraint {
    /// The minimum number of items, if constrained.
    min_items: Option<u32>,
    /// The maximum number of items, if constrained.
    max_items: Option<u32>,
    /// Whether sequence elements must be unique.
    unique_items: bool,
}

impl SequenceConstraint {
    /// Creates sequence constraints from item limits and uniqueness semantics.
    ///
    /// # Parameters
    ///
    /// * `min_items` - The optional minimum item count.
    /// * `max_items` - The optional maximum item count.
    /// * `unique_items` - Whether sequence elements must be unique.
    ///
    /// # Panics
    ///
    /// Panics when the supplied minimum item count exceeds the maximum.
    ///
    /// # Returns
    ///
    /// Sequence constraints containing the supplied limits and uniqueness
    /// policy.
    #[must_use]
    pub const fn new(
        min_items: Option<u32>,
        max_items: Option<u32>,
        unique_items: bool,
    ) -> Self {
        if let (Some(min_items), Some(max_items)) = (min_items, max_items) {
            assert!(
                min_items <= max_items,
                "minimum item count cannot exceed maximum item count"
            );
        }
        Self {
            min_items,
            max_items,
            unique_items,
        }
    }

    /// Returns the minimum number of items, if constrained.
    ///
    /// # Returns
    ///
    /// `Some` with the minimum item count when constrained; otherwise, `None`.
    #[must_use]
    #[inline(always)]
    pub const fn min_items(self) -> Option<u32> {
        self.min_items
    }

    /// Returns the maximum number of items, if constrained.
    ///
    /// # Returns
    ///
    /// `Some` with the maximum item count when constrained; otherwise, `None`.
    #[must_use]
    #[inline(always)]
    pub const fn max_items(self) -> Option<u32> {
        self.max_items
    }

    /// Returns whether sequence elements must be unique.
    ///
    /// # Returns
    ///
    /// `true` when sequence elements must be unique; otherwise, `false`.
    #[must_use]
    #[inline(always)]
    pub const fn has_unique_items(self) -> bool {
        self.unique_items
    }
}
