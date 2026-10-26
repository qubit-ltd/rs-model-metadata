// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_reflect::value::DynamicMut;
use qubit_reflect::value::DynamicOwned;
use qubit_reflect::value::DynamicRef;
use qubit_reflect::value::Local;

/// A shared borrow of a property value with its concrete type erased.
///
/// The borrow remains tied to the lifetime of the underlying property value,
/// so it cannot outlive that value.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyBorrowed;
///
/// let value = 42_u32;
/// let property = PropertyBorrowed::new(&value);
/// assert!(property.is::<u32>());
/// ```
pub type PropertyBorrowed<'a> = DynamicRef<'a, Local>;

/// An exclusive borrow of a property value with its concrete type erased.
///
/// The borrow remains tied to the lifetime of the underlying property value,
/// and permits mutation only while the exclusive borrow is held.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyMutable;
///
/// let mut value = 42_u32;
/// let mut property = PropertyMutable::new(&mut value);
/// *property.downcast_mut::<u32>().expect("the stored type is u32") = 7;
/// drop(property);
/// assert_eq!(value, 7);
/// ```
pub type PropertyMutable<'a> = DynamicMut<'a, Local>;

/// An owned property value with its concrete type erased.
///
/// The value is stored locally and can be recovered when its concrete type is
/// known.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyOwned;
///
/// let property = PropertyOwned::new(42_u32);
/// assert_eq!(property.downcast::<u32>().expect("the stored type is u32"), 42);
/// ```
pub type PropertyOwned = DynamicOwned<Local>;
