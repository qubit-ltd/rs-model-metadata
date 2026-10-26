// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_reflect::value::Local;
use qubit_reflect::value::ValueForRef;

use super::PropertyBorrowed;
use super::internal::property_slice::PropertySlice;
use super::internal::typed_property_slice::TypedPropertySlice;

/// A lifetime-preserving, type-erased borrowed slice returned by a property.
///
/// # Type Parameters
///
/// * `'a` - The lifetime of the borrowed slice and every element returned by
///   [`get`](Self::get) or [`iter`](Self::iter).
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::BorrowedPropertySlice;
///
///
/// let values = [3_u32, 5];
/// let slice = BorrowedPropertySlice::new(&values);
/// assert_eq!(slice.len(), 2);
/// assert!(slice.get(0).is_some());
/// ```
#[must_use]
pub struct BorrowedPropertySlice<'a> {
    /// The lifetime-preserving erased slice implementation.
    value: Box<dyn PropertySlice<'a> + 'a>,
}

impl<'a> BorrowedPropertySlice<'a> {
    /// Erases a borrowed slice without extending its lifetime.
    ///
    /// # Type Parameters
    ///
    /// * `T` - A reflected element type that supports local shared borrowing.
    ///
    /// # Parameters
    ///
    /// * `value` - The slice whose elements remain borrowed for `'a`.
    ///
    /// # Returns
    ///
    /// A type-erased adapter that preserves the input slice's lifetime.
    #[doc(hidden)]
    pub fn new<T: ValueForRef<Local>>(value: &'a [T]) -> Self {
        Self {
            value: Box::new(TypedPropertySlice::new(value)),
        }
    }

    /// Returns the number of elements.
    ///
    /// # Returns
    ///
    /// The number of elements in the retained slice.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.value.len()
    }

    /// Returns whether the slice contains no elements.
    ///
    /// # Returns
    ///
    /// `true` when the retained slice has no elements.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns one element as a borrowed reflected value.
    ///
    /// # Parameters
    ///
    /// * `index` - The zero-based element position.
    ///
    /// # Returns
    ///
    /// `Some` with the borrowed element when `index` is in bounds, or `None`
    /// otherwise.
    #[inline]
    pub fn get(&self, index: usize) -> Option<PropertyBorrowed<'a>> {
        self.value.get(index)
    }

    /// Returns a borrowing iterator over every element.
    pub fn iter(&self) -> Box<dyn Iterator<Item = PropertyBorrowed<'a>> + 'a> {
        self.value.iter()
    }
}
