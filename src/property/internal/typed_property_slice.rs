// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_reflect::value::Local;
use qubit_reflect::value::ValueForRef;

use super::super::PropertyBorrowed;
use super::property_slice::PropertySlice;

/// Implements [`PropertySlice`] for a slice with one concrete element type.
pub(in crate::property) struct TypedPropertySlice<'a, T> {
    /// The typed slice retained by the erased adapter.
    values: &'a [T],
}

impl<'a, T> TypedPropertySlice<'a, T> {
    /// Creates the typed implementation retained by the erased slice.
    ///
    /// # Parameters
    ///
    /// * `values` - The values borrowed for `'a` and exposed through the erased
    ///   slice adapter.
    ///
    /// # Returns
    ///
    /// A typed adapter that retains the input slice borrow for `'a`.
    #[must_use]
    #[inline]
    pub(in crate::property) fn new(values: &'a [T]) -> Self {
        Self { values }
    }
}

impl<'a, T: ValueForRef<Local>> PropertySlice<'a> for TypedPropertySlice<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    fn get(&self, index: usize) -> Option<PropertyBorrowed<'a>> {
        self.values.get(index).map(PropertyBorrowed::new)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = PropertyBorrowed<'a>> + 'a> {
        Box::new(self.values.iter().map(PropertyBorrowed::new))
    }
}
