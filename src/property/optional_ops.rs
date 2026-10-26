// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::any::TypeId;

use qubit_reflect::descriptor::Reflect;
use qubit_reflect::value::Local;
use qubit_reflect::value::ValueForMut;
use qubit_reflect::value::ValueForRef;

use super::PropertyBorrowed;
use super::PropertyMutable;
use super::PropertyTypeMismatch;

/// Generated exact type operations for traversing an `Option<T>` property.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::OptionalOps;
/// use qubit_model_metadata::metadata::PropertyBorrowed;
///
/// let value = Some(42_u32);
/// let ops = OptionalOps::for_element::<u32>();
/// let projected = ops
///     .project_ref(PropertyBorrowed::new(&value))
///     .expect("the property contains an Option<u32>");
/// let element = projected.expect("the option contains a value");
/// assert_eq!(*element.downcast::<u32>().expect("the element type is u32"), 42);
///
/// let absent: Option<u32> = None;
/// assert!(ops
///     .project_ref(PropertyBorrowed::new(&absent))
///     .expect("the property contains an Option<u32>")
///     .is_none());
/// ```
#[must_use = "optional operations are useful only when used to project a property"]
#[derive(Clone, Copy)]
pub struct OptionalOps {
    /// Produces the exact reflected element type identity.
    element_type_id: fn() -> TypeId,
    /// Projects a shared `Option<T>` borrow while validating its concrete type.
    project_ref: for<'a> fn(PropertyBorrowed<'a>) -> Result<Option<PropertyBorrowed<'a>>, PropertyTypeMismatch>,
    /// Projects a mutable `Option<T>` borrow while validating its concrete
    /// type.
    project_mut: for<'a> fn(PropertyMutable<'a>) -> Result<Option<PropertyMutable<'a>>, PropertyTypeMismatch>,
}

impl OptionalOps {
    /// Builds projection operations for one reflected element type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The reflected element type stored by `Option<T>`.
    ///
    /// # Returns
    ///
    /// Operations that validate and project shared or mutable local borrows
    /// of `Option<T>`.
    #[doc(hidden)]
    pub fn for_element<T: Reflect + ValueForRef<Local> + ValueForMut<Local> + std::any::Any>() -> Self {
        /// Returns the exact `TypeId` of the reflected element type.
        fn type_id<T: 'static>() -> TypeId {
            TypeId::of::<T>()
        }
        /// Projects a matching optional value into a shared element borrow.
        fn project_ref<'a, T>(value: PropertyBorrowed<'a>) -> Result<Option<PropertyBorrowed<'a>>, PropertyTypeMismatch>
        where
            T: Reflect + ValueForRef<Local> + std::any::Any,
        {
            match value.downcast::<Option<T>>() {
                Ok(value) => Ok(value.as_ref().map(PropertyBorrowed::new_any)),
                Err(failure) => {
                    let (error, _) = failure.into_parts();
                    Err(PropertyTypeMismatch {
                        expected: error.expected(),
                        actual: error.actual(),
                    })
                }
            }
        }
        /// Projects a matching optional value into a mutable element borrow.
        fn project_mut<'a, T>(value: PropertyMutable<'a>) -> Result<Option<PropertyMutable<'a>>, PropertyTypeMismatch>
        where
            T: Reflect + ValueForMut<Local>,
        {
            match value.downcast::<Option<T>>() {
                Ok(value) => Ok(value.as_mut().map(PropertyMutable::new)),
                Err(failure) => {
                    let (error, _) = failure.into_parts();
                    Err(PropertyTypeMismatch {
                        expected: error.expected(),
                        actual: error.actual(),
                    })
                }
            }
        }
        Self {
            element_type_id: type_id::<T>,
            project_ref: project_ref::<T>,
            project_mut: project_mut::<T>,
        }
    }

    /// Returns the exact Rust identity of `T`.
    #[must_use]
    #[inline]
    pub fn element_type_id(self) -> TypeId {
        (self.element_type_id)()
    }

    /// Projects the contained value as a shared borrow.
    ///
    /// # Parameters
    ///
    /// * `value` - A borrow created from the property value to inspect.
    ///
    /// # Returns
    ///
    /// `Ok(Some(_))` when the property is `Some(T)`, `Ok(None)` when it is
    /// `None`, and an error when the borrowed property is not `Option<T>`.
    ///
    /// # Errors
    ///
    /// Returns `PropertyTypeMismatch` when the property's concrete type does
    /// not match the element type used to create these operations.
    #[inline]
    pub fn project_ref<'a>(
        self,
        value: PropertyBorrowed<'a>,
    ) -> Result<Option<PropertyBorrowed<'a>>, PropertyTypeMismatch> {
        (self.project_ref)(value)
    }

    /// Projects the contained value as a mutable borrow.
    ///
    /// # Parameters
    ///
    /// * `value` - An exclusive borrow created from the property value to
    ///   mutate.
    ///
    /// # Returns
    ///
    /// `Ok(Some(_))` when the property is `Some(T)`, `Ok(None)` when it is
    /// `None`, and an error when the borrowed property is not `Option<T>`.
    ///
    /// # Errors
    ///
    /// Returns `PropertyTypeMismatch` when the property's concrete type does
    /// not match the element type used to create these operations.
    #[inline]
    pub fn project_mut<'a>(
        self,
        value: PropertyMutable<'a>,
    ) -> Result<Option<PropertyMutable<'a>>, PropertyTypeMismatch> {
        (self.project_mut)(value)
    }
}
