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
use qubit_reflect::value::ValueForRef;

use super::GetterOutputKind;
use super::PropertyAccessError;
use super::PropertyBorrowed;
use super::PropertyValue;

/// A lifetime-preserving local getter adapter.
///
/// The function receives a borrowed target and returns a borrowed or owned
/// erased property value.
///
/// # Examples
///
/// ```
/// use qubit_reflect::value::Local;
/// use qubit_model_metadata::metadata::GetterAdapter;
/// use qubit_model_metadata::metadata::PropertyAccessError;
/// use qubit_model_metadata::metadata::PropertyBorrowed;
/// use qubit_model_metadata::metadata::PropertyOwned;
/// use qubit_model_metadata::metadata::PropertyValue;
/// fn get_count<'a>(_: PropertyBorrowed<'a>) -> Result<PropertyValue<'a>, PropertyAccessError> {
///     Ok(PropertyValue::Owned(PropertyOwned::new(3_u32)))
/// }
///
/// let adapter: GetterAdapter = get_count;
/// assert!(adapter(PropertyBorrowed::new(&())).is_ok());
/// ```
pub type GetterAdapter = for<'a> fn(PropertyBorrowed<'a>) -> Result<PropertyValue<'a>, PropertyAccessError>;

/// Metadata and adapter for one explicit getter method.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::GetterMetadata;
/// use qubit_model_metadata::metadata::GetterOutputKind;
/// use qubit_model_metadata::metadata::PropertyAccessError;
/// use qubit_model_metadata::metadata::PropertyBorrowed;
/// use qubit_model_metadata::metadata::PropertyValue;
/// use qubit_reflect::Reflect;
///
/// # mod example {
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::PropertyAccessError;
/// use qubit_model_metadata::metadata::PropertyBorrowed;
/// use qubit_model_metadata::metadata::PropertyValue;
/// use qubit_reflect::Reflect;
/// #[Model]
/// pub struct Item {
///     pub value: u32,
/// }
///
/// pub fn get_value<'a>(
///     target: PropertyBorrowed<'a>,
/// ) -> Result<PropertyValue<'a>, PropertyAccessError> {
///     let item = target.downcast::<Item>().ok().expect("validated Item target");
///     Ok(PropertyValue::Borrowed(PropertyBorrowed::new(&item.value)))
/// }
/// # }
///
/// let value_type = qubit_model_metadata::metadata::TypeMetadata::of::<example::Item>()
///     .field("value")
///     .expect("value field")
///     .type_expression_data()
///     .expect("field type");
/// let getter = GetterMetadata::new::<example::Item>(
///     "value",
///     value_type,
///     std::any::TypeId::of::<u32>,
///     GetterOutputKind::Borrowed,
///     example::get_value,
/// );
/// assert_eq!(getter.rust_method_name(), "value");
/// ```
#[derive(Clone, Copy)]
pub struct GetterMetadata {
    /// The Rust identifier of the generated getter method.
    rust_method_name: &'static str,
    /// The declared type of the getter output.
    output_type: &'static TypeExpressionData,
    /// Produces the exact Rust identity of the declared output type.
    output_type_id: fn() -> TypeId,
    /// Produces the exact element identity for a borrowed-slice result.
    output_element_type_id: Option<fn() -> TypeId>,
    /// Whether the getter result borrows from the target or owns its value.
    output_kind: GetterOutputKind,
    /// Produces the exact Rust type identity accepted by the getter.
    target_type_id: fn() -> TypeId,
    /// Executes the generated getter after type validation.
    adapter: GetterAdapter,
}

impl GetterMetadata {
    /// Creates getter metadata for methods on `Target`.
    ///
    /// # Type Parameters
    ///
    /// * `Target` - The exact Rust target type accepted by the getter.
    ///
    /// # Parameters
    ///
    /// * `rust_method_name` - The source-level getter method name.
    /// * `output_type` - The declared reflection type of the getter result.
    /// * `output_type_id` - Function that returns the exact Rust output type
    ///   identity.
    /// * `output_kind` - Whether the getter borrows or owns its result.
    /// * `adapter` - The function that invokes the generated getter.
    ///
    /// # Returns
    ///
    /// Metadata that validates targets before invoking `adapter`.
    #[must_use]
    #[inline]
    pub const fn new<Target: 'static>(
        rust_method_name: &'static str,
        output_type: &'static TypeExpressionData,
        output_type_id: fn() -> TypeId,
        output_kind: GetterOutputKind,
        adapter: GetterAdapter,
    ) -> Self {
        Self {
            rust_method_name,
            output_type,
            output_type_id,
            output_element_type_id: None,
            output_kind,
            target_type_id: TypeId::of::<Target>,
            adapter,
        }
    }

    /// Records the exact element identity for a borrowed-slice getter.
    ///
    /// # Parameters
    ///
    /// * `output_element_type_id` - Function that returns the concrete slice
    ///   element identity.
    ///
    /// # Returns
    ///
    /// This metadata with the element identity recorded.
    #[must_use]
    #[inline]
    pub fn with_output_element_type_id(mut self, output_element_type_id: fn() -> TypeId) -> Self {
        self.output_element_type_id = Some(output_element_type_id);
        self
    }

    /// Returns the Rust getter method name.
    ///
    /// # Returns
    ///
    /// The source-level getter method name.
    #[must_use]
    #[inline]
    pub const fn rust_method_name(&self) -> &'static str {
        self.rust_method_name
    }

    /// Returns the declared getter output type.
    ///
    /// # Returns
    ///
    /// The reflection type declared for the getter result.
    #[must_use]
    #[inline]
    pub const fn output_type_data(&self) -> &'static TypeExpressionData {
        self.output_type
    }

    /// Binds this getter's source-level output expression to a reflection
    /// context.
    ///
    /// # Parameters
    ///
    /// * `context` - Context used to resolve nested type metadata.
    ///
    /// # Returns
    ///
    /// A read-only expression view bound to `context`.
    #[must_use]
    #[inline]
    pub fn output_type_in<'ctx>(&self, context: &'ctx ReflectionContext) -> TypeExpression<'ctx> {
        type_expression_in(self.output_type, context)
    }

    /// Returns the process-local Rust identity of the declared output type.
    ///
    /// # Returns
    ///
    /// The `TypeId` used to distinguish the exact declared output type.
    #[must_use]
    #[inline]
    pub fn output_type_id(&self) -> TypeId {
        (self.output_type_id)()
    }

    /// Returns the exact element identity when this getter yields a slice.
    ///
    /// # Returns
    ///
    /// `Some` contains the slice element's `TypeId`; `None` means no element
    /// identity was recorded for this getter.
    #[inline]
    pub fn output_element_type_id(&self) -> Option<TypeId> {
        self.output_element_type_id.map(|type_id| type_id())
    }

    /// Returns whether the getter borrows or owns its output.
    ///
    /// # Returns
    ///
    /// The getter's declared output ownership and borrowing mode.
    #[must_use]
    #[inline]
    pub const fn output_kind(&self) -> GetterOutputKind {
        self.output_kind
    }

    /// Returns the exact Rust type identity accepted by this getter.
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

    /// Executes this getter after exact target validation.
    ///
    /// # Parameters
    ///
    /// * `target` - The borrowed target value to pass to the getter adapter.
    ///
    /// # Returns
    ///
    /// The getter result, preserving whether its value is borrowed or owned.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyAccessError::TargetTypeMismatch`] when `target` has
    /// a different concrete type, or propagates the generated adapter error.
    #[inline]
    pub fn get<'a, T: ValueForRef<Local>>(&self, target: &'a T) -> Result<PropertyValue<'a>, PropertyAccessError> {
        self.get_dynamic(PropertyBorrowed::new(target))
    }

    /// Executes this getter using a dynamically borrowed target.
    ///
    /// # Parameters
    ///
    /// * `target` - Dynamically borrowed target passed to the getter adapter.
    ///
    /// # Returns
    ///
    /// The getter output, preserving whether it borrows from `target` or owns
    /// its value.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyAccessError::TargetTypeMismatch`] if the target has a
    /// different concrete type, or propagates the generated adapter error.
    #[inline]
    pub fn get_dynamic<'a>(&self, target: PropertyBorrowed<'a>) -> Result<PropertyValue<'a>, PropertyAccessError> {
        (self.adapter)(target)
    }
}

impl fmt::Debug for GetterMetadata {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GetterMetadata")
            .field("rust_method_name", &self.rust_method_name)
            .field("output_kind", &self.output_kind)
            .finish_non_exhaustive()
    }
}
