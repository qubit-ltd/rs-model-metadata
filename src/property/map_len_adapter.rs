// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::PropertyAccessError;
use super::PropertyValue;

/// A callback that reads a map length; `None` means an optional map field is
/// absent.
///
/// The adapter returns `Ok(None)` when the property value is an absent optional
/// map, and `Ok(Some(length))` for a present map.
///
/// # Parameters
///
/// * `value` - The erased value returned by the property's getter.
///
/// # Returns
///
/// `Ok(Some(length))` contains the number of entries in a present map.
/// `Ok(None)` indicates that an optional map is absent.
///
/// # Errors
///
/// Returns [`PropertyAccessError`] when the getter value cannot be read as the
/// expected map type.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
///
/// use qubit_model_metadata::metadata::MapLenAdapter;
/// use qubit_model_metadata::metadata::PropertyOwned;
/// use qubit_model_metadata::metadata::PropertyValue;
///
/// let map_len: MapLenAdapter = |value| {
///     let map = match value {
///         PropertyValue::Owned(value) => Some(
///             value
///                 .downcast_ref::<HashMap<String, u32>>()
///                 .expect("the property contains the expected map type"),
///         ),
///         _ => None,
///     };
///     Ok(map.map(HashMap::len))
/// };
///
/// let map = HashMap::from([(String::from("key"), 2_u32)]);
/// let value = PropertyValue::Owned(PropertyOwned::new(map));
/// assert_eq!(map_len(&value).expect("map length"), Some(1));
/// ```
pub type MapLenAdapter = for<'a> fn(&PropertyValue<'a>) -> Result<Option<usize>, PropertyAccessError>;
