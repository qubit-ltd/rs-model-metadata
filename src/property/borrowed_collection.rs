// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::PropertyBorrowed;

/// A borrowed iterator over members selected from one collection value.
///
/// The `'a` lifetime ties every yielded [`PropertyBorrowed`] item to the
/// collection value inspected by the adapter that produced this iterator.
/// Neither the iterator nor its items can outlive that value.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::BorrowedCollection;
///
/// // A collection returned by a `CollectionItemsAdapter` can be inspected
/// // while its source value remains borrowed.
/// fn remaining_length<'a>(collection: &BorrowedCollection<'a>) -> Option<usize> {
///     collection.remaining_len()
/// }
/// ```
#[must_use]
pub struct BorrowedCollection<'a> {
    /// The erased iterator that retains each selected member's borrow lifetime.
    values: Box<dyn Iterator<Item = PropertyBorrowed<'a>> + 'a>,
}

impl<'a> BorrowedCollection<'a> {
    /// Erases an iterator while retaining the lifetime of its borrowed items.
    ///
    /// # Parameters
    ///
    /// * `values` - The iterator whose items remain borrowed for `'a`.
    ///
    /// # Returns
    ///
    /// A collection adapter that preserves the iterator's item lifetime.
    #[doc(hidden)]
    #[inline]
    pub fn new(values: impl Iterator<Item = PropertyBorrowed<'a>> + 'a) -> Self {
        Self {
            values: Box::new(values),
        }
    }

    /// Returns the iterator's exact remaining length when it is known.
    ///
    /// Returns `None` when the iterator's size hint does not establish an
    /// exact length; otherwise returns the number of remaining items, including
    /// `Some(0)` when it is exhausted.
    #[inline]
    pub fn remaining_len(&self) -> Option<usize> {
        let (lower, upper) = self.values.size_hint();
        (upper == Some(lower)).then_some(lower)
    }
}

impl<'a> Iterator for BorrowedCollection<'a> {
    type Item = PropertyBorrowed<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.values.next()
    }
}
