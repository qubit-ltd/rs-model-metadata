// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::super::PropertyBorrowed;

/// Erases a borrowed slice while preserving the lifetime of each element.
///
/// # Type Parameters
///
/// * `'a` - The lifetime shared by the borrowed slice elements and yielded
///   property references.
pub(in crate::property) trait PropertySlice<'a> {
    /// Returns the number of values in the borrowed slice.
    ///
    /// # Returns
    ///
    /// The number of elements in the slice.
    #[must_use]
    fn len(&self) -> usize;
    /// Returns the value at the requested index, or `None` when it is out of
    /// bounds.
    ///
    /// # Parameters
    ///
    /// * `index` - The zero-based element position to retrieve.
    ///
    /// # Returns
    ///
    /// The borrowed element at `index`, or `None` if `index` is outside the
    /// slice.
    fn get(&self, index: usize) -> Option<PropertyBorrowed<'a>>;
    /// Iterates over borrowed elements without cloning them.
    ///
    /// # Returns
    ///
    /// A lazy iterator yielding each borrowed element in slice order.
    #[must_use = "consume the iterator to visit the borrowed values"]
    fn iter(&self) -> Box<dyn Iterator<Item = PropertyBorrowed<'a>> + 'a>;
}
