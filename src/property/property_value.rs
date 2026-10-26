// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_reflect::InvocationOutput;
use qubit_reflect::descriptor::BorrowOrigin;
use qubit_reflect::value::Local;

use super::BorrowedPropertySlice;
use super::PropertyBorrowed;
use super::PropertyOwned;

/// A property getter result that either borrows from its target or owns a
/// value.
///
/// # Type Parameters
///
/// * `'a` - The lifetime shared by borrowed getter results and their target.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyOwned;
/// use qubit_model_metadata::metadata::PropertyValue;
///
/// let value = PropertyValue::Owned(PropertyOwned::new(42_u32));
/// let output = value.into_invocation_output();
/// assert!(output.into_owned::<u32>().is_ok());
/// ```
#[must_use]
pub enum PropertyValue<'a> {
    /// A dynamically typed borrow tied to the target.
    Borrowed(PropertyBorrowed<'a>),
    /// An optional dynamically typed borrow.
    OptionalBorrowed(Option<PropertyBorrowed<'a>>),
    /// A lifetime-preserving borrowed slice.
    BorrowedSlice(BorrowedPropertySlice<'a>),
    /// An owned dynamically typed value.
    Owned(PropertyOwned),
}

impl<'a> PropertyValue<'a> {
    /// Converts a property getter result into the shared reflection invocation
    /// output contract without erasing optionality or borrowed slices.
    ///
    /// Property getter borrows can only originate from their target, so every
    /// borrowed output records [`BorrowOrigin::Receiver`]. Borrowed slices are
    /// explicitly materialized in O(n) time into per-element borrow wrappers;
    /// the elements themselves are not cloned. This allocates output storage
    /// and origin metadata. Direct [`BorrowedPropertySlice::get`] avoids that
    /// materialization, although constructing the slice adapter also boxes it.
    ///
    /// # Panics
    ///
    /// Panics if the erased slice reports a length containing an index that it
    /// cannot return. Implementations created by this crate preserve that
    /// invariant.
    ///
    /// # Returns
    ///
    /// The reflection invocation output containing the original borrowed or
    /// owned value representation.
    pub fn into_invocation_output(self) -> InvocationOutput<'a, Local> {
        let receiver_origin = || Box::new([BorrowOrigin::Receiver]);
        match self {
            Self::Borrowed(value) => InvocationOutput::Ref {
                value,
                origins: receiver_origin(),
            },
            Self::OptionalBorrowed(value) => InvocationOutput::OptionalRef {
                value,
                origins: receiver_origin(),
            },
            Self::BorrowedSlice(value) => {
                let values = (0..value.len())
                    .map(|index| {
                        value
                            .get(index)
                            .expect("indices below the reported slice length must exist")
                    })
                    .collect();
                InvocationOutput::RefSlice {
                    values,
                    origins: receiver_origin(),
                }
            }
            Self::Owned(value) => InvocationOutput::Owned(value),
        }
    }
}
