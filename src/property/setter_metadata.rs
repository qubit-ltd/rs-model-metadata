// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::any::TypeId;
use std::fmt;

use qubit_reflect::__private::codegen_v4::expression::TypeExpressionData;
use qubit_reflect::__private::codegen_v4::expression::type_expression_in;
use qubit_reflect::ReflectionContext;
use qubit_reflect::descriptor::TypeExpression;
use qubit_reflect::value::Local;
use qubit_reflect::value::ValueFor;
use qubit_reflect::value::ValueForMut;

use super::PropertyAccessError;
use super::PropertyMutable;
use super::PropertyOwned;
use super::PropertySetFailure;
use super::PropertyTypeMismatch;

/// A local setter adapter with recoverable pre-execution failure.
///
/// The adapter returns a [`PropertySetFailure`] that can retain the replacement
/// when validation fails before ownership crosses the execution boundary.
///
/// # Examples
///
/// ```
/// use qubit_reflect::value::Local;
/// use qubit_model_metadata::metadata::PropertySetFailure;
/// use qubit_model_metadata::metadata::PropertyMutable;
/// use qubit_model_metadata::metadata::PropertyOwned;
/// use qubit_model_metadata::metadata::SetterAdapter;
///
///
/// fn set_value(_: PropertyMutable<'_>, _: PropertyOwned) -> Result<(), PropertySetFailure> {
///     Ok(())
/// }
///
/// let adapter: SetterAdapter = set_value;
/// assert!(adapter(PropertyMutable::new(&mut 1_u32), PropertyOwned::new(2_u32)).is_ok());
/// ```
pub type SetterAdapter = fn(PropertyMutable<'_>, PropertyOwned) -> Result<(), PropertySetFailure>;

/// Validates a setter target and replacement before ownership crosses the
/// adapter execution boundary.
///
/// Target identity is checked before replacement identity, matching the
/// recovery error precedence used by the setter API.
fn validate_setter_types(
    target: &PropertyMutable<'_>,
    value: &PropertyOwned,
    expected_target: TypeId,
    expected_value: TypeId,
) -> Result<(), PropertyAccessError> {
    let actual_target = target.type_id();
    if actual_target != expected_target {
        return Err(PropertyAccessError::TargetTypeMismatch(PropertyTypeMismatch {
            expected: expected_target,
            actual: actual_target,
        }));
    }

    let actual_value = value.type_id();
    if actual_value != expected_value {
        return Err(PropertyAccessError::ValueTypeMismatch(PropertyTypeMismatch {
            expected: expected_value,
            actual: actual_value,
        }));
    }

    Ok(())
}

/// Metadata and adapter for one explicit setter method.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_reflect::value::Local;
/// use qubit_model_metadata::metadata::SetterMetadata;
/// use qubit_model_metadata::metadata::PropertyMutable;
/// use qubit_model_metadata::metadata::PropertyOwned;
/// use qubit_reflect::Reflect;
///
///
/// # mod example {
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::PropertySetFailure;
/// use qubit_model_metadata::metadata::PropertyMutable;
/// use qubit_model_metadata::metadata::PropertyOwned;
/// use qubit_reflect::Reflect;
///
///
/// #[Model]
/// pub struct Item { pub value: u32 }
///
/// pub fn set_value(target: PropertyMutable<'_>, value: PropertyOwned) -> Result<(), PropertySetFailure> {
///     let mut item = target.downcast::<Item>().ok().expect("validated Item target");
///     item.value = value.downcast::<u32>().ok().expect("validated u32 value");
///     Ok(())
/// }
/// # }
///
/// let value_type = qubit_model_metadata::metadata::TypeMetadata::of::<example::Item>()
///     .field("value").expect("value field").type_expression_data().expect("field type");
/// let setter = SetterMetadata::new::<example::Item, u32>("set_value", value_type, example::set_value);
/// assert_eq!(setter.rust_method_name(), "set_value");
/// ```
#[derive(Clone, Copy)]
pub struct SetterMetadata {
    /// The Rust identifier of the generated setter method.
    rust_method_name: &'static str,
    /// The declared type accepted by the setter.
    input_type: &'static TypeExpressionData,
    /// Produces the exact Rust type identity accepted for the target.
    target_type_id: fn() -> TypeId,
    /// Produces the exact Rust type identity accepted for the input.
    input_type_id: fn() -> TypeId,
    /// Executes the generated setter after type validation.
    adapter: SetterAdapter,
}

impl SetterMetadata {
    /// Creates setter metadata for `Target` accepting exact `Input` values.
    ///
    /// # Type Parameters
    ///
    /// * `Target` - The exact Rust target type accepted by the setter.
    /// * `Input` - The exact Rust replacement type accepted by the setter.
    ///
    /// # Parameters
    ///
    /// * `rust_method_name` - The source-level setter method name.
    /// * `input_type` - The declared reflection type accepted by the setter.
    /// * `adapter` - The function that invokes the generated setter.
    ///
    /// # Returns
    ///
    /// Metadata that validates both target and replacement before invocation.
    #[must_use]
    #[inline]
    pub const fn new<Target: 'static, Input: 'static>(
        rust_method_name: &'static str,
        input_type: &'static TypeExpressionData,
        adapter: SetterAdapter,
    ) -> Self {
        Self {
            rust_method_name,
            input_type,
            target_type_id: TypeId::of::<Target>,
            input_type_id: TypeId::of::<Input>,
            adapter,
        }
    }

    /// Returns the Rust setter method name.
    ///
    /// # Returns
    ///
    /// The source-level setter method name.
    #[must_use]
    #[inline]
    pub const fn rust_method_name(&self) -> &'static str {
        self.rust_method_name
    }

    /// Returns the exact setter input type.
    ///
    /// # Returns
    ///
    /// The reflection type declared for the setter input.
    #[must_use]
    #[inline]
    pub const fn input_type_data(&self) -> &'static TypeExpressionData {
        self.input_type
    }

    /// Binds this setter's declared input expression to `context`.
    ///
    /// # Parameters
    ///
    /// * `context` - The reflection context used to resolve nested type
    ///   metadata.
    ///
    /// # Returns
    ///
    /// The setter input expression bound to `context`.
    #[inline]
    pub fn input_type_in<'ctx>(&self, context: &'ctx ReflectionContext) -> TypeExpression<'ctx> {
        type_expression_in(self.input_type, context)
    }

    /// Returns the exact Rust type identity accepted for this setter target.
    ///
    /// # Returns
    ///
    /// The target type's `TypeId` used for exact adapter validation.
    #[doc(hidden)]
    #[must_use]
    #[inline]
    pub fn target_type_id(&self) -> TypeId {
        (self.target_type_id)()
    }

    /// Returns the exact Rust type identity accepted for this setter input.
    ///
    /// # Returns
    ///
    /// The input type's `TypeId` used for exact adapter validation.
    #[doc(hidden)]
    #[must_use]
    #[inline]
    pub fn input_type_id(&self) -> TypeId {
        (self.input_type_id)()
    }

    /// Executes this setter after exact target and value validation.
    ///
    /// # Parameters
    ///
    /// * `target` - The mutable target that receives the replacement.
    /// * `value` - The owned replacement value.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the setter succeeds, or a recoverable failure describing
    /// whether the replacement is still available.
    ///
    /// # Errors
    ///
    /// Returns [`PropertySetFailure`] retaining `value` when target or input
    /// validation fails before adapter execution, or reports the adapter error.
    pub fn set<T: ValueForMut<Local>, V: ValueFor<Local>>(
        &self,
        target: &mut T,
        value: V,
    ) -> Result<(), PropertySetFailure> {
        self.set_dynamic(PropertyMutable::new(target), PropertyOwned::new(value))
    }

    /// Executes this setter using a dynamic target and owned replacement.
    ///
    /// # Parameters
    ///
    /// * `target` - The dynamically typed mutable target to update.
    /// * `value` - The owned replacement value passed to the setter adapter.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the adapter succeeds; otherwise, a failure describing
    /// whether the replacement remains available for recovery.
    ///
    /// # Errors
    ///
    /// Returns [`PropertySetFailure`] with `value` retained when target or
    /// input validation fails before adapter execution. Adapter failures carry
    /// the recovery state produced by the setter.
    pub fn set_dynamic(&self, target: PropertyMutable<'_>, value: PropertyOwned) -> Result<(), PropertySetFailure> {
        if let Err(error) = validate_setter_types(&target, &value, (self.target_type_id)(), (self.input_type_id)()) {
            return Err(PropertySetFailure::before_execution(error, value));
        }
        (self.adapter)(target, value)
    }
}

impl fmt::Debug for SetterMetadata {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SetterMetadata")
            .field("rust_method_name", &self.rust_method_name)
            .finish_non_exhaustive()
    }
}
