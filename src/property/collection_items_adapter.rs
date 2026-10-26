// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::BorrowedCollection;
use super::PropertyAccessError;
use super::PropertyValue;
use crate::metadata::SelectorPosition;

/// Reads selected members from a collection while preserving their borrow
/// lifetime.
///
/// The callback receives the erased collection value and the structural
/// position of the selected members. It must return borrows tied to the input
/// value, so selected members cannot outlive the collection.
///
/// # Type Parameters
///
/// * `'a` - The lifetime shared by the input value and every returned member.
///
/// # Parameters
///
/// * `value` - The exact borrowed collection value to inspect.
/// * `position` - Whether to select sequence elements, map keys, or map values.
///
/// # Returns
///
/// `Ok(Some(collection))` when the optional collection is present, where the
/// iterator yields its selected members; `Ok(None)` when an optional
/// collection is absent.
///
/// # Errors
///
/// Returns [`PropertyAccessError`] when the value cannot be read as the
/// expected collection type.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::BorrowedCollection;
/// use qubit_model_metadata::metadata::PropertyAccessError;
/// use qubit_model_metadata::metadata::PropertyOwned;
/// use qubit_model_metadata::metadata::PropertyValue;
/// use qubit_model_metadata::metadata::SelectorPosition;
///
/// fn read_members<'a>(
///     _value: &PropertyValue<'a>,
///     _position: SelectorPosition,
/// ) -> Result<Option<BorrowedCollection<'a>>, PropertyAccessError> {
///     Ok(None)
/// }
///
/// let adapter = read_members;
/// let value = PropertyValue::Owned(PropertyOwned::new(()));
/// let members = adapter(&value, SelectorPosition::Element)
///     .expect("the collection is readable");
/// assert!(members.is_none());
/// ```
pub type CollectionItemsAdapter =
    for<'a> fn(&PropertyValue<'a>, SelectorPosition) -> Result<Option<BorrowedCollection<'a>>, PropertyAccessError>;
