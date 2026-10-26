// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::fmt;

use qubit_reflect::__private::codegen_v4::expression::TypeExpressionData;
use qubit_reflect::__private::codegen_v4::expression::type_expression_in;
use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::FieldSetFailure;
use qubit_reflect::descriptor::TypeExpression;
use qubit_reflect::error::ContextError;
use qubit_reflect::value::DynamicOwned;
use qubit_reflect::value::Local;
use qubit_reflect::value::ValueFor;
use qubit_reflect::value::ValueForMut;
use qubit_reflect::value::ValueForRef;

use super::GetterMetadata;
use super::OptionalOps;
use super::PropertyAccessError;
use super::PropertyBorrowed;
use super::PropertyMutable;
use super::PropertyOwned;
use super::PropertySetFailure;
use super::PropertyStorageKind;
use super::PropertyValue;
use super::SetterMetadata;
use crate::metadata::FieldMetadata;
#[cfg(feature = "validation")]
use crate::type_metadata::TypeMetadata;

/// Merged field/getter/setter metadata for one model property.
///
/// Instances are provided by the public property metadata flow, such as
/// [`TypeMetadata::try_properties`](crate::metadata::TypeMetadata::try_properties)
/// followed by property lookup.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::PropertyMetadata;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// # mod example {
/// use qubit_model_derive::Model;
///
/// #[Model(id = "example.PropertyMetadataNote")]
/// pub struct Note { pub title: String }
/// # }
///
/// let metadata = TypeMetadata::of::<example::Note>();
/// let properties = metadata.try_properties().expect("property metadata");
/// let title: &PropertyMetadata = properties.property("title").expect("title property");
/// assert_eq!(title.name(), "title");
/// ```
#[derive(Clone, Copy)]
pub struct PropertyMetadata {
    /// The public property name used by model metadata.
    name: &'static str,
    /// The declared property type.
    type_expression: Option<&'static TypeExpressionData>,
    /// The reflected backing field, when one exists.
    field: Option<&'static FieldMetadata>,
    /// The explicit getter adapter, when one exists.
    getter: Option<&'static GetterMetadata>,
    /// The explicit setter adapter, when one exists.
    setter: Option<&'static SetterMetadata>,
}

impl fmt::Debug for PropertyMetadata {
    /// Formats the metadata's public name and the presence of its components.
    ///
    /// # Parameters
    ///
    /// * `formatter` - The destination formatter.
    ///
    /// # Returns
    ///
    /// The result of writing the debug representation.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PropertyMetadata")
            .field("name", &self.name)
            .field("has_type_expression", &self.type_expression.is_some())
            .field("has_field", &self.field.is_some())
            .field("has_getter", &self.getter.is_some())
            .field("has_setter", &self.setter.is_some())
            .finish()
    }
}

impl PropertyMetadata {
    /// Creates merged property metadata.
    ///
    /// # Parameters
    ///
    /// * `name` - The property's public metadata name.
    /// * `type_expression` - The declared property type.
    /// * `field` - The optional reflected backing field.
    /// * `getter` - The optional explicit getter adapter.
    /// * `setter` - The optional explicit setter adapter.
    ///
    /// # Returns
    ///
    /// Metadata that combines the property's field and explicit accessors.
    #[must_use]
    #[inline]
    pub(crate) const fn new(
        name: &'static str,
        type_expression: Option<&'static TypeExpressionData>,
        field: Option<&'static FieldMetadata>,
        getter: Option<&'static GetterMetadata>,
        setter: Option<&'static SetterMetadata>,
    ) -> Self {
        Self {
            name,
            type_expression,
            field,
            getter,
            setter,
        }
    }

    /// Returns the public property name.
    ///
    /// # Returns
    ///
    /// The property name used by model metadata.
    #[must_use]
    #[inline]
    pub const fn name(&self) -> &'static str {
        self.name
    }
    /// Returns this property's type expression bound to `context`.
    ///
    /// A backing field determines the expression for field-backed properties.
    /// Otherwise the explicit declaration or accessor signature is used.
    ///
    /// # Parameters
    ///
    /// * `context` - The reflection context used to bind the expression.
    ///
    /// # Returns
    ///
    /// `Some` with the first available expression from the backing field,
    /// declared property type, getter output, or setter input; `None` when
    /// none of those sources provides a type expression.
    #[inline]
    pub fn type_expression_in<'ctx>(&self, context: &'ctx ReflectionContext) -> Option<TypeExpression<'ctx>> {
        if let Some(field) = self.field
            && let Some(expression) = field.type_expression_in(context)
        {
            return Some(expression);
        }
        if let Some(expression) = self.type_expression {
            return Some(type_expression_in(expression, context));
        }
        if let Some(getter) = self.getter {
            return Some(getter.output_type_in(context));
        }
        self.setter.map(|setter| setter.input_type_in(context))
    }

    /// Resolves this property's concrete type in `context`, when available.
    ///
    /// # Returns
    ///
    /// `Some` with the concrete descriptor when resolved, or `None` for
    /// symbolic and opaque types.
    ///
    /// # Errors
    ///
    /// Returns the context lookup error if resolving the reflected type fails.
    #[inline]
    pub fn descriptor_in<'ctx>(
        &self,
        context: &'ctx ReflectionContext,
    ) -> Result<Option<TypeDescriptor<'ctx>>, ContextError> {
        if let Some(field) = self.field
            && let Some(reflected) = field.reflect_in(context)
        {
            return Ok(reflected.find_concrete_type());
        }
        let type_id = self
            .getter
            .map(GetterMetadata::output_type_id)
            .or_else(|| self.setter.map(SetterMetadata::input_type_id));
        type_id.map_or(Ok(None), |type_id| context.find_type_by_id(type_id))
    }

    /// Returns the backing field, or `None` for computed and virtual
    /// properties.
    ///
    /// # Returns
    ///
    /// `Some` with the reflected backing field, or `None` for computed and
    /// virtual properties.
    #[inline]
    pub const fn field(&self) -> Option<&'static FieldMetadata> {
        self.field
    }
    /// Returns generated projection operations for an optional backing field.
    ///
    /// # Returns
    ///
    /// The field's optional projection operations, or `None` when this
    /// property has no backing field or the field has no such operations.
    #[inline]
    pub fn optional_ops(&self) -> Option<OptionalOps> {
        self.field.and_then(FieldMetadata::optional_ops)
    }
    /// Returns the explicit getter, or `None` when reads use field fallback.
    ///
    /// # Returns
    ///
    /// `Some` with the explicit getter, or `None` when reads fall back to the
    /// backing field.
    #[inline]
    pub const fn getter(&self) -> Option<&'static GetterMetadata> {
        self.getter
    }
    /// Returns the explicit setter, or `None` when writes use field fallback.
    ///
    /// # Returns
    ///
    /// `Some` with the explicit setter, or `None` when writes fall back to the
    /// backing field.
    #[inline]
    pub const fn setter(&self) -> Option<&'static SetterMetadata> {
        self.setter
    }
    /// Returns whether a reflected field backs this property.
    ///
    /// # Returns
    ///
    /// `true` when a reflected field backs the property.
    #[must_use]
    #[inline]
    pub const fn is_field(&self) -> bool {
        self.field.is_some()
    }
    /// Returns whether an explicit getter is present.
    ///
    /// # Returns
    ///
    /// `true` when the property has an explicit getter.
    #[must_use]
    #[inline]
    pub const fn is_getter(&self) -> bool {
        self.getter.is_some()
    }
    /// Returns whether an explicit setter is present.
    ///
    /// # Returns
    ///
    /// `true` when the property has an explicit setter.
    #[must_use]
    #[inline]
    pub const fn is_setter(&self) -> bool {
        self.setter.is_some()
    }
    /// Returns whether the property can be read.
    ///
    /// # Returns
    ///
    /// `true` when the property has a backing field or explicit getter.
    #[must_use]
    #[inline]
    pub const fn is_readable(&self) -> bool {
        self.is_field() || self.is_getter()
    }
    /// Returns whether the property can be written.
    ///
    /// # Returns
    ///
    /// `true` when the property has a backing field or explicit setter.
    #[must_use]
    #[inline]
    pub const fn is_writable(&self) -> bool {
        self.is_field() || self.is_setter()
    }
    /// Returns whether the property is getter-only and computed.
    ///
    /// # Returns
    ///
    /// `true` when an explicit getter exists without a backing field.
    #[must_use]
    #[inline]
    pub const fn is_computed(&self) -> bool {
        !self.is_field() && self.is_getter()
    }

    /// Returns the storage classification.
    ///
    /// # Returns
    ///
    /// `FieldBacked` when a field is present, `Computed` when only a getter
    /// provides the value, or `Virtual` otherwise.
    #[inline]
    #[must_use]
    pub const fn storage_kind(&self) -> PropertyStorageKind {
        if self.is_field() {
            PropertyStorageKind::FieldBacked
        } else if self.is_getter() {
            PropertyStorageKind::Computed
        } else {
            PropertyStorageKind::Virtual
        }
    }

    /// Reads with explicit getter precedence over field fallback.
    ///
    /// # Type Parameters
    ///
    /// * `T` - A local value reference supported by the reflection adapter.
    ///
    /// # Parameters
    ///
    /// * `target` - The borrowed model instance to read.
    ///
    /// # Returns
    ///
    /// The value supplied by the getter or backing field.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyAccessError::NotReadable`] when neither a getter nor
    /// a backing field exists, and otherwise propagates access failures.
    ///
    /// # Panics
    ///
    /// Panics if metadata contains a backing field without a concrete
    /// reflection descriptor. Generated runtime metadata cannot create this
    /// state.
    pub fn get<'a, T: ValueForRef<Local>>(&self, target: &'a T) -> Result<PropertyValue<'a>, PropertyAccessError> {
        self.get_dynamic(PropertyBorrowed::new(target))
    }

    /// Reads this property using a dynamically borrowed target.
    ///
    /// A getter takes precedence over a backing field. Returns
    /// [`PropertyAccessError::NotReadable`] when neither is available, and
    /// otherwise propagates access failures while preserving borrowed or
    /// owned output.
    ///
    /// # Parameters
    ///
    /// * `target` - The model instance to read through a dynamic borrow.
    ///
    /// # Returns
    ///
    /// The value supplied by the getter or backing field.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyAccessError::NotReadable`] when neither source is
    /// available, or propagates the selected accessor's failure.
    pub fn get_dynamic<'a>(&self, target: PropertyBorrowed<'a>) -> Result<PropertyValue<'a>, PropertyAccessError> {
        self.get_dynamic_in(ReflectionContext::global(), target)
    }

    /// Reads this property using descriptors bound to `context`.
    ///
    /// # Parameters
    ///
    /// * `context` - The reflection context used to resolve the backing field.
    /// * `target` - The model instance to read through a dynamic borrow.
    ///
    /// # Returns
    ///
    /// The value supplied by the getter or backing field.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyAccessError::NotReadable`] when neither source is
    /// available, or propagates the selected accessor's failure.
    pub fn get_dynamic_in<'a>(
        &self,
        context: &ReflectionContext,
        target: PropertyBorrowed<'a>,
    ) -> Result<PropertyValue<'a>, PropertyAccessError> {
        if let Some(getter) = self.getter {
            return getter.get_dynamic(target);
        }
        if let Some(field) = self.field {
            return field
                .reflect_in(context)
                .expect("runtime properties only reference concrete fields")
                .get_dynamic(target)
                .map(PropertyValue::Borrowed)
                .map_err(Into::into);
        }
        Err(PropertyAccessError::NotReadable)
    }

    /// Reads a field-backed property after bypassing reflection visibility.
    ///
    /// # Safety
    ///
    /// The caller must ensure access to this property is authorized by the
    /// application's privacy and encapsulation invariants.
    ///
    /// # Parameters
    ///
    /// * `target` - The model instance to read through a dynamic borrow.
    ///
    /// # Returns
    ///
    /// The value supplied by the getter or backing field.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyAccessError::NotReadable`] when neither source is
    /// available, or propagates the selected accessor's failure.
    pub unsafe fn get_dynamic_unsafe<'a>(
        &self,
        target: PropertyBorrowed<'a>,
    ) -> Result<PropertyValue<'a>, PropertyAccessError> {
        unsafe { self.get_dynamic_unsafe_in(ReflectionContext::global(), target) }
    }

    /// Reads this property with a context-bound descriptor after bypassing
    /// reflection visibility.
    ///
    /// # Safety
    ///
    /// The caller must ensure access to the field is permitted by the
    /// application's privacy and encapsulation invariants. The descriptor
    /// still enforces field readability and exact target identity.
    ///
    /// # Parameters
    ///
    /// * `context` - The reflection context used to resolve the backing field.
    /// * `target` - The model instance to read through a dynamic borrow.
    ///
    /// # Returns
    ///
    /// The value supplied by the getter or backing field.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyAccessError::NotReadable`] when no readable source
    /// exists, or propagates the selected accessor's failure.
    pub unsafe fn get_dynamic_unsafe_in<'a>(
        &self,
        context: &ReflectionContext,
        target: PropertyBorrowed<'a>,
    ) -> Result<PropertyValue<'a>, PropertyAccessError> {
        if let Some(getter) = self.getter {
            return getter.get_dynamic(target);
        }
        if let Some(field) = self.field {
            let Some(reflected) = field.reflect_in(context) else {
                return Err(PropertyAccessError::NotReadable);
            };
            return unsafe { reflected.get_dynamic_unsafe(target) }
                .map(PropertyValue::Borrowed)
                .map_err(Into::into);
        }
        Err(PropertyAccessError::NotReadable)
    }

    /// Reads a property using its statically known owner descriptor.
    ///
    /// # Safety
    ///
    /// The caller must ensure access to the field is permitted by the
    /// application's privacy and encapsulation invariants. The descriptor
    /// still enforces field readability and exact target identity.
    ///
    /// # Parameters
    ///
    /// * `owner` - The type metadata that owns the reflected field.
    /// * `context` - The reflection context used to bind its descriptor.
    /// * `target` - The model instance to read through a dynamic borrow.
    ///
    /// # Returns
    ///
    /// The value supplied by the getter or owner-bound field descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyAccessError::NotReadable`] when no readable source
    /// exists, or propagates the selected accessor's failure.
    #[cfg(feature = "validation")]
    pub(crate) unsafe fn get_dynamic_unsafe_for<'a>(
        &self,
        owner: &'static TypeMetadata,
        context: &ReflectionContext,
        target: PropertyBorrowed<'a>,
    ) -> Result<PropertyValue<'a>, PropertyAccessError> {
        if let Some(getter) = self.getter {
            return getter.get_dynamic(target);
        }
        if let Some(field) = self.field {
            let Some(reflected) = owner.field_descriptor_in(field, context) else {
                return Err(PropertyAccessError::NotReadable);
            };
            return unsafe { reflected.get_dynamic_unsafe(target) }
                .map(PropertyValue::Borrowed)
                .map_err(Into::into);
        }
        Err(PropertyAccessError::NotReadable)
    }

    /// Writes with explicit setter precedence over field fallback.
    ///
    /// # Type Parameters
    ///
    /// * `T` - A mutable local value reference supported by the adapter.
    /// * `V` - An owned local value accepted by the setter or field.
    ///
    /// # Parameters
    ///
    /// * `target` - The mutable model instance to update.
    /// * `value` - The owned replacement value.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the write succeeds; otherwise a failure that retains the
    /// replacement if execution had not started.
    ///
    /// # Errors
    ///
    /// Returns [`PropertySetFailure`] retaining the replacement when no write
    /// operation has started, and otherwise reports the setter or field error.
    ///
    /// # Panics
    ///
    /// Panics if metadata contains a backing field without a concrete
    /// reflection descriptor. Generated runtime metadata cannot create this
    /// state.
    pub fn set<T: ValueForMut<Local>, V: ValueFor<Local>>(
        &self,
        target: &mut T,
        value: V,
    ) -> Result<(), PropertySetFailure> {
        self.set_dynamic(PropertyMutable::new(target), PropertyOwned::new(value))
    }

    /// Writes an owned replacement using a dynamic mutable target.
    ///
    /// A setter takes precedence over a backing field. Returns
    /// [`PropertySetFailure`] with the replacement retained when execution
    /// has not started, or with the underlying setter or field failure after
    /// execution begins.
    ///
    /// # Parameters
    ///
    /// * `target` - The model instance to update through a dynamic mutable
    ///   borrow.
    /// * `value` - The owned replacement value.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the write succeeds, or a failure carrying error and
    /// recovery information.
    ///
    /// # Errors
    ///
    /// Returns [`PropertySetFailure`] when no writable source exists or the
    /// selected setter or field rejects the replacement.
    pub fn set_dynamic(&self, target: PropertyMutable<'_>, value: PropertyOwned) -> Result<(), PropertySetFailure> {
        self.set_dynamic_in(ReflectionContext::global(), target, value)
    }

    /// Writes using descriptors bound to `context`.
    ///
    /// # Parameters
    ///
    /// * `context` - The reflection context used to resolve the backing field.
    /// * `target` - The model instance to update through a dynamic mutable
    ///   borrow.
    /// * `value` - The owned replacement value.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the write succeeds, or a failure carrying error and
    /// recovery information.
    ///
    /// # Errors
    ///
    /// Returns [`PropertySetFailure`] when no writable source exists or the
    /// selected setter or field rejects the replacement.
    pub fn set_dynamic_in(
        &self,
        context: &ReflectionContext,
        target: PropertyMutable<'_>,
        value: PropertyOwned,
    ) -> Result<(), PropertySetFailure> {
        if let Some(setter) = self.setter {
            return setter.set_dynamic(target, value);
        }
        if let Some(field) = self.field {
            return field
                .reflect_in(context)
                .expect("runtime properties only reference concrete fields")
                .set_dynamic(target, value)
                .map_err(Self::map_field_set_failure);
        }
        Err(PropertySetFailure::before_execution(
            PropertyAccessError::NotWritable,
            value,
        ))
    }

    /// Writes using descriptors bound to `context`, bypassing field visibility.
    ///
    /// # Safety
    ///
    /// The caller must ensure mutation through this property is authorized by
    /// the application's privacy and encapsulation invariants.
    ///
    /// # Parameters
    ///
    /// * `context` - The reflection context used to resolve the backing field.
    /// * `target` - The model instance to update through a dynamic mutable
    ///   borrow.
    /// * `value` - The owned replacement value.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the write succeeds, or a failure carrying error and
    /// recovery information.
    ///
    /// # Errors
    ///
    /// Returns [`PropertySetFailure`] when no writable source exists or the
    /// selected setter or field rejects the replacement.
    pub unsafe fn set_dynamic_unsafe_in(
        &self,
        context: &ReflectionContext,
        target: PropertyMutable<'_>,
        value: PropertyOwned,
    ) -> Result<(), PropertySetFailure> {
        if let Some(setter) = self.setter {
            return setter.set_dynamic(target, value);
        }
        if let Some(field) = self.field {
            let Some(reflected) = field.reflect_in(context) else {
                return Err(PropertySetFailure::before_execution(
                    PropertyAccessError::NotWritable,
                    value,
                ));
            };
            return unsafe { reflected.set_dynamic_unsafe(target, value) }.map_err(Self::map_field_set_failure);
        }
        Err(PropertySetFailure::before_execution(
            PropertyAccessError::NotWritable,
            value,
        ))
    }

    /// Writes a field-backed property after bypassing reflection visibility.
    ///
    /// # Safety
    ///
    /// The caller must ensure mutation through this property is authorized by
    /// the application's privacy and encapsulation invariants.
    ///
    /// # Parameters
    ///
    /// * `target` - The model instance to update through a dynamic mutable
    ///   borrow.
    /// * `value` - The owned replacement value.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the write succeeds, or a failure carrying error and
    /// recovery information.
    ///
    /// # Errors
    ///
    /// Returns [`PropertySetFailure`] when no writable source exists or the
    /// selected setter or field rejects the replacement.
    pub unsafe fn set_dynamic_unsafe(
        &self,
        target: PropertyMutable<'_>,
        value: PropertyOwned,
    ) -> Result<(), PropertySetFailure> {
        unsafe { self.set_dynamic_unsafe_in(ReflectionContext::global(), target, value) }
    }

    /// Returns the retained type-expression data for generated metadata.
    ///
    /// # Returns
    ///
    /// The declared, getter-output, or setter-input expression data, in that
    /// order of precedence; `None` when no source provides an expression.
    #[doc(hidden)]
    pub(crate) const fn type_expression_data(&self) -> Option<&'static TypeExpressionData> {
        if let Some(expression) = self.type_expression {
            return Some(expression);
        }
        if let Some(getter) = self.getter {
            return Some(getter.output_type_data());
        }
        match self.setter {
            Some(setter) => Some(setter.input_type_data()),
            None => None,
        }
    }

    /// Returns the explicitly declared type expression bound to `context`.
    ///
    /// # Parameters
    ///
    /// * `context` - The reflection context to bind the declared expression to.
    ///
    /// # Returns
    ///
    /// The explicitly declared expression bound to `context`, or `None` when
    /// no declared expression is present.
    pub(crate) fn declared_type_expression_in<'ctx>(
        &self,
        context: &'ctx ReflectionContext,
    ) -> Option<TypeExpression<'ctx>> {
        self.type_expression
            .map(|expression| type_expression_in(expression, context))
    }

    /// Resolves a field-backed property's descriptor from its known owner.
    ///
    /// # Parameters
    ///
    /// * `owner` - The type metadata that owns the reflected field.
    /// * `context` - The reflection context used to resolve its descriptor.
    ///
    /// # Returns
    ///
    /// The resolved concrete descriptor, or `None` when the type is symbolic
    /// or opaque.
    ///
    /// # Errors
    ///
    /// Returns the context lookup error if resolving the reflected type fails.
    #[cfg(feature = "validation")]
    pub(crate) fn descriptor_in_owner<'ctx>(
        &self,
        owner: &'static TypeMetadata,
        context: &'ctx ReflectionContext,
    ) -> Result<Option<TypeDescriptor<'ctx>>, ContextError> {
        if let Some(field) = self.field
            && self.getter.is_none()
            && let Some(reflected) = owner.field_descriptor_in(field, context)
        {
            return Ok(reflected.find_concrete_type());
        }
        self.descriptor_in(context)
    }

    /// Converts a rejected reflected field write into the property recovery
    /// contract without changing ownership of the replacement.
    ///
    /// # Parameters
    ///
    /// * `failure` - The rejected field write and its retained replacement.
    ///
    /// # Returns
    ///
    /// The equivalent property-level failure with the same recovery value.
    fn map_field_set_failure(failure: FieldSetFailure<DynamicOwned<Local>>) -> PropertySetFailure {
        let (error, recovery) = failure.into_parts();
        PropertySetFailure::from_field_set_failure(error, recovery)
    }
}
