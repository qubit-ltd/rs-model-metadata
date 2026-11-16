// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::PropertyAccessError;
use super::PropertyBorrowed;

/// Compares two exact borrowed collection elements.
///
/// The adapter reports type or access failures through [`PropertyAccessError`].
///
/// # Examples
///
/// ```
/// use qubit_reflect::value::Local;
/// use qubit_model_metadata::metadata::PropertyAccessError;
/// use qubit_model_metadata::metadata::PropertyBorrowed;
///
/// fn equal(_: PropertyBorrowed<'_>, _: PropertyBorrowed<'_>) -> Result<bool, PropertyAccessError> {
///     Ok(true)
/// }
///
/// assert!(equal(PropertyBorrowed::new(&1_u32), PropertyBorrowed::new(&1_u32)).expect("comparison"));
/// ```
pub type ItemEqAdapter = for<'a> fn(PropertyBorrowed<'a>, PropertyBorrowed<'a>) -> Result<bool, PropertyAccessError>;
