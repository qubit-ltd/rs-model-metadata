// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Constraints that apply to an ordered sequence.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::SequenceConstraint;
///
/// let constraint = SequenceConstraint::new(Some(1), Some(10), true);
/// assert_eq!(constraint.max_items(), Some(10));
/// assert!(constraint.unique_items());
/// ```
#[derive(Clone, Copy, Debug)]
pub struct SequenceConstraint {
    /// The minimum number of items, if constrained.
    min_items: Option<usize>,
    /// The maximum number of items, if constrained.
    max_items: Option<usize>,
    /// Whether sequence elements must be unique.
    unique_items: bool,
    /// Optional non-recursive element semantics.
    element: Option<&'static crate::field_semantics::SelectorMetadata>,
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
    /// # Returns
    ///
    /// Sequence constraints containing the supplied limits and uniqueness
    /// policy.
    ///
    /// # Panics
    ///
    /// Panics when the supplied minimum item count exceeds the maximum.
    #[must_use]
    pub const fn new(min_items: Option<usize>, max_items: Option<usize>, unique_items: bool) -> Self {
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
            element: None,
        }
    }

    /// Attaches non-recursive element semantics.
    #[must_use]
    pub const fn with_element(mut self, element: &'static crate::field_semantics::SelectorMetadata) -> Self {
        self.element = Some(element);
        self
    }

    /// Returns the minimum number of items, if constrained.
    ///
    /// # Returns
    ///
    /// `Some` with the minimum item count when constrained; otherwise, `None`.
    #[must_use]
    #[inline(always)]
    pub const fn min_items(&self) -> Option<usize> {
        self.min_items
    }

    /// Returns the maximum number of items, if constrained.
    ///
    /// # Returns
    ///
    /// `Some` with the maximum item count when constrained; otherwise, `None`.
    #[must_use]
    #[inline(always)]
    pub const fn max_items(&self) -> Option<usize> {
        self.max_items
    }

    /// Returns whether sequence elements must be unique.
    ///
    /// # Returns
    ///
    /// `true` when sequence elements must be unique; otherwise, `false`.
    #[must_use]
    #[inline(always)]
    pub const fn unique_items(&self) -> bool {
        self.unique_items
    }

    /// Returns element semantics, if declared.
    #[must_use]
    pub const fn element(&self) -> Option<&'static crate::field_semantics::SelectorMetadata> {
        self.element
    }
}
