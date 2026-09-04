// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow multiple-public-types
//! Safe local erased adapters for model properties.

use std::any::TypeId;

use qubit_reflect::FieldAccessError;
use qubit_reflect::FieldSetRecovery;
use qubit_reflect::InvocationOutput;
use qubit_reflect::ReflectedMut;
use qubit_reflect::ReflectedOwned;
use qubit_reflect::ReflectedRef;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::TypeMismatch;
use qubit_reflect::descriptor::TypeRef;
use qubit_reflect::invoke::BorrowOrigin;
use qubit_reflect::value::Local;

use crate::FieldMetadata;

/// Classifies how a property stores or computes its value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PropertyStorageKind {
    /// A reflected field stores the value.
    FieldBacked,
    /// A getter computes the value.
    Computed,
    /// Only an explicit setter is available.
    Virtual,
}

/// A property getter result that either borrows from its target or owns a
/// value.
#[must_use]
pub enum PropertyValue<'a> {
    /// A dynamically typed borrow tied to the target.
    Borrowed(ReflectedRef<'a>),
    /// An optional dynamically typed borrow.
    OptionalBorrowed(Option<ReflectedRef<'a>>),
    /// A lifetime-preserving borrowed slice.
    BorrowedSlice(BorrowedPropertySlice<'a>),
    /// An owned dynamically typed value.
    Owned(ReflectedOwned),
}

impl<'a> PropertyValue<'a> {
    /// Converts a property getter result into the shared reflection invocation
    /// output contract without erasing optionality or borrowed slices.
    ///
    /// Property getter borrows can only originate from their target, so every
    /// borrowed output records [`BorrowOrigin::Receiver`].
    #[must_use]
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

trait PropertySlice<'a> {
    /// Returns the number of values in the borrowed slice.
    fn len(&self) -> usize;
    /// Returns the value at `index`, or `None` when it is out of bounds.
    fn get(&self, index: usize) -> Option<ReflectedRef<'a>>;
}

struct TypedPropertySlice<'a, T> {
    /// The typed slice retained by the erased adapter.
    values: &'a [T],
}

impl<'a, T: 'static> PropertySlice<'a> for TypedPropertySlice<'a, T> {
    fn len(&self) -> usize {
        self.values.len()
    }

    fn get(&self, index: usize) -> Option<ReflectedRef<'a>> {
        self.values.get(index).map(ReflectedRef::new)
    }
}

/// A lifetime-preserving, type-erased borrowed slice returned by a property.
#[must_use]
pub struct BorrowedPropertySlice<'a> {
    /// The lifetime-preserving erased slice implementation.
    value: Box<dyn PropertySlice<'a> + 'a>,
}

impl<'a> BorrowedPropertySlice<'a> {
    /// Erases a borrowed slice without extending its lifetime.
    #[doc(hidden)]
    pub fn new<T: 'static>(value: &'a [T]) -> Self {
        Self {
            value: Box::new(TypedPropertySlice { values: value }),
        }
    }

    /// Returns the number of elements.
    #[must_use]
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.value.len()
    }

    /// Returns whether the slice contains no elements.
    #[must_use]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns one element as a borrowed reflected value.
    #[must_use]
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<ReflectedRef<'a>> {
        self.value.get(index)
    }
}

/// Distinguishes lifetime-preserving borrowed output from owned output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GetterOutputKind {
    /// The output borrows from its target.
    Borrowed,
    /// The output owns its value.
    Owned,
}

/// A lifetime-preserving local getter adapter.
pub type GetterAdapter = for<'a> fn(ReflectedRef<'a>) -> Result<PropertyValue<'a>, PropertyAccessError>;

/// A local setter adapter with recoverable pre-execution failure.
pub type SetterAdapter = fn(ReflectedMut<'_>, ReflectedOwned) -> Result<(), PropertySetFailure>;

/// A property operation failed before or during adapter execution.
#[must_use]
#[derive(Debug, thiserror::Error)]
pub enum PropertyAccessError {
    /// The target has a different concrete type.
    #[error("property target type mismatch")]
    TargetTypeMismatch(TypeMismatch),
    /// The replacement has a different concrete type.
    #[error("property value type mismatch")]
    ValueTypeMismatch(TypeMismatch),
    /// The property has no readable source.
    #[error("property is not readable")]
    NotReadable,
    /// The property has no writable destination.
    #[error("property is not writable")]
    NotWritable,
    /// Generated adapter code is unavailable.
    #[error("property adapter is unavailable")]
    AdapterUnavailable,
    /// The underlying reflection field operation failed.
    #[error("reflected field access failed: {0}")]
    Field(#[from] FieldAccessError),
    /// An adapter reported a static user-facing error.
    #[error("property method failed: {0}")]
    User(&'static str),
}

impl PropertyAccessError {
    /// Creates an adapter-defined user-method error.
    #[must_use = "handle the property access error"]
    pub const fn user(message: &'static str) -> Self {
        Self::User(message)
    }
}

/// A property set failure with optional untouched replacement ownership.
#[must_use]
pub struct PropertySetFailure {
    /// The structured reason why the property write failed.
    error: Box<PropertyAccessError>,
    /// The untouched replacement retained for a pre-execution failure.
    replacement: Option<Box<ReflectedOwned>>,
}

impl PropertySetFailure {
    /// Creates a pre-execution failure retaining the replacement.
    #[doc(hidden)]
    #[must_use = "handle the property set failure"]
    pub fn before_execution(error: PropertyAccessError, replacement: ReflectedOwned) -> Self {
        Self {
            error: Box::new(error),
            replacement: Some(Box::new(replacement)),
        }
    }

    /// Creates an adapter failure after ownership crossed the execution
    /// boundary.
    #[doc(hidden)]
    #[must_use = "handle the property set failure"]
    pub fn after_execution(error: PropertyAccessError) -> Self {
        Self {
            error: Box::new(error),
            replacement: None,
        }
    }

    /// Returns the structured failure.
    #[must_use = "inspect the property failure before discarding it"]
    #[inline(always)]
    pub const fn error(&self) -> &PropertyAccessError {
        &self.error
    }

    /// Returns the untouched replacement for pre-execution failure.
    #[must_use]
    pub fn replacement(&self) -> Option<&ReflectedOwned> {
        self.replacement.as_deref()
    }

    /// Consumes the failure and returns its parts.
    #[must_use = "handle the property error and recovered replacement"]
    pub fn into_parts(self) -> (PropertyAccessError, Option<ReflectedOwned>) {
        (*self.error, self.replacement.map(|replacement| *replacement))
    }
}

impl core::fmt::Debug for PropertySetFailure {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PropertySetFailure")
            .field("error", &self.error)
            .field("has_replacement", &self.replacement.is_some())
            .finish()
    }
}

impl core::fmt::Display for PropertySetFailure {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.error.fmt(formatter)
    }
}

impl std::error::Error for PropertySetFailure {}

/// Metadata and adapter for one explicit getter method.
#[derive(Clone, Copy)]
pub struct GetterMetadata {
    /// The Rust identifier of the generated getter method.
    rust_method_name: &'static str,
    /// The declared type of the getter output.
    output_type: &'static TypeRef,
    /// Whether the getter result borrows from the target or owns its value.
    output_kind: GetterOutputKind,
    /// Produces the exact Rust type identity accepted by the getter.
    target_type_id: fn() -> TypeId,
    /// Executes the generated getter after type validation.
    adapter: GetterAdapter,
}

impl GetterMetadata {
    /// Creates getter metadata for methods on `Target`.
    #[must_use]
    pub const fn new<Target: 'static>(
        rust_method_name: &'static str,
        output_type: &'static TypeRef,
        output_kind: GetterOutputKind,
        adapter: GetterAdapter,
    ) -> Self {
        Self {
            rust_method_name,
            output_type,
            output_kind,
            target_type_id: TypeId::of::<Target>,
            adapter,
        }
    }

    /// Returns the Rust getter method name.
    #[must_use]
    #[inline(always)]
    pub const fn rust_method_name(&self) -> &'static str {
        self.rust_method_name
    }
    /// Returns the declared getter output type.
    #[must_use]
    #[inline(always)]
    pub const fn output_type(&self) -> &'static TypeRef {
        self.output_type
    }
    /// Returns whether the getter borrows or owns its output.
    #[must_use]
    #[inline(always)]
    pub const fn output_kind(&self) -> GetterOutputKind {
        self.output_kind
    }

    /// Returns the exact Rust type identity accepted by this getter.
    #[doc(hidden)]
    #[must_use]
    pub fn target_type_id(&self) -> TypeId {
        (self.target_type_id)()
    }

    /// Executes this getter after exact target validation.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyAccessError::TargetTypeMismatch`] when `target` has
    /// a different concrete type, or propagates the generated adapter error.
    #[must_use = "handle property access failure"]
    pub fn get<'a>(&self, target: ReflectedRef<'a>) -> Result<PropertyValue<'a>, PropertyAccessError> {
        let actual = reflected_ref_type_id(&target);
        let expected = (self.target_type_id)();
        if actual != expected {
            return Err(PropertyAccessError::TargetTypeMismatch(TypeMismatch::new(
                expected, actual,
            )));
        }
        (self.adapter)(target)
    }
}

impl core::fmt::Debug for GetterMetadata {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("GetterMetadata")
            .field("rust_method_name", &self.rust_method_name)
            .field("output_kind", &self.output_kind)
            .finish_non_exhaustive()
    }
}

/// Metadata and adapter for one explicit setter method.
#[derive(Clone, Copy)]
pub struct SetterMetadata {
    /// The Rust identifier of the generated setter method.
    rust_method_name: &'static str,
    /// The declared type accepted by the setter.
    input_type: &'static TypeRef,
    /// Produces the exact Rust type identity accepted for the target.
    target_type_id: fn() -> TypeId,
    /// Produces the exact Rust type identity accepted for the input.
    input_type_id: fn() -> TypeId,
    /// Executes the generated setter after type validation.
    adapter: SetterAdapter,
}

impl SetterMetadata {
    /// Creates setter metadata for `Target` accepting exact `Input` values.
    #[must_use]
    pub const fn new<Target: 'static, Input: 'static>(
        rust_method_name: &'static str,
        input_type: &'static TypeRef,
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
    #[must_use]
    #[inline(always)]
    pub const fn rust_method_name(&self) -> &'static str {
        self.rust_method_name
    }
    /// Returns the exact setter input type.
    #[must_use]
    #[inline(always)]
    pub const fn input_type(&self) -> &'static TypeRef {
        self.input_type
    }

    /// Returns the exact Rust type identity accepted for this setter target.
    #[doc(hidden)]
    #[must_use]
    pub fn target_type_id(&self) -> TypeId {
        (self.target_type_id)()
    }

    /// Returns the exact Rust type identity accepted for this setter input.
    #[doc(hidden)]
    #[must_use]
    pub fn input_type_id(&self) -> TypeId {
        (self.input_type_id)()
    }

    /// Executes this setter after exact target and value validation.
    ///
    /// # Errors
    ///
    /// Returns [`PropertySetFailure`] retaining `value` when target or input
    /// validation fails before adapter execution, or reports the adapter error.
    #[must_use = "handle property write failure and recover the replacement when available"]
    pub fn set(&self, target: ReflectedMut<'_>, value: ReflectedOwned) -> Result<(), PropertySetFailure> {
        let actual_target = reflected_mut_type_id(&target);
        let expected_target = (self.target_type_id)();
        if actual_target != expected_target {
            return Err(PropertySetFailure::before_execution(
                PropertyAccessError::TargetTypeMismatch(TypeMismatch::new(expected_target, actual_target)),
                value,
            ));
        }
        let actual_value = reflected_owned_type_id(&value);
        let expected_value = (self.input_type_id)();
        if actual_value != expected_value {
            return Err(PropertySetFailure::before_execution(
                PropertyAccessError::ValueTypeMismatch(TypeMismatch::new(expected_value, actual_value)),
                value,
            ));
        }
        (self.adapter)(target, value)
    }
}

impl core::fmt::Debug for SetterMetadata {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("SetterMetadata")
            .field("rust_method_name", &self.rust_method_name)
            .finish_non_exhaustive()
    }
}

/// Merged field/getter/setter metadata for one model property.
#[derive(Clone, Copy, Debug)]
pub struct PropertyMetadata {
    /// The public property name used by model metadata.
    name: &'static str,
    /// The declared property type.
    type_ref: &'static TypeRef,
    /// The reflected backing field, when one exists.
    field: Option<&'static FieldMetadata>,
    /// The explicit getter adapter, when one exists.
    getter: Option<&'static GetterMetadata>,
    /// The explicit setter adapter, when one exists.
    setter: Option<&'static SetterMetadata>,
}

impl PropertyMetadata {
    /// Creates merged property metadata.
    #[must_use]
    pub(crate) const fn new(
        name: &'static str,
        type_ref: &'static TypeRef,
        field: Option<&'static FieldMetadata>,
        getter: Option<&'static GetterMetadata>,
        setter: Option<&'static SetterMetadata>,
    ) -> Self {
        Self {
            name,
            type_ref,
            field,
            getter,
            setter,
        }
    }

    /// Returns the public property name.
    #[must_use]
    #[inline(always)]
    pub const fn name(&self) -> &'static str {
        self.name
    }
    /// Returns the property type reference.
    #[must_use]
    #[inline(always)]
    pub const fn type_ref(&self) -> &'static TypeRef {
        self.type_ref
    }
    /// Returns the resolved property type descriptor, or `None` for symbolic
    /// and opaque property types.
    #[must_use]
    #[inline(always)]
    pub const fn descriptor(&self) -> Option<&'static TypeDescriptor> {
        self.type_ref.as_resolved()
    }
    /// Returns the backing field, or `None` for computed and virtual
    /// properties.
    #[must_use]
    #[inline(always)]
    pub const fn field(&self) -> Option<&'static FieldMetadata> {
        self.field
    }
    /// Returns the explicit getter, or `None` when reads use field fallback.
    #[must_use]
    #[inline(always)]
    pub const fn getter(&self) -> Option<&'static GetterMetadata> {
        self.getter
    }
    /// Returns the explicit setter, or `None` when writes use field fallback.
    #[must_use]
    #[inline(always)]
    pub const fn setter(&self) -> Option<&'static SetterMetadata> {
        self.setter
    }
    /// Returns whether a reflected field backs this property.
    #[must_use]
    pub const fn is_field(&self) -> bool {
        self.field.is_some()
    }
    /// Returns whether an explicit getter is present.
    #[must_use]
    pub const fn is_getter(&self) -> bool {
        self.getter.is_some()
    }
    /// Returns whether an explicit setter is present.
    #[must_use]
    pub const fn is_setter(&self) -> bool {
        self.setter.is_some()
    }
    /// Returns whether the property can be read.
    #[must_use]
    pub const fn is_readable(&self) -> bool {
        self.is_field() || self.is_getter()
    }
    /// Returns whether the property can be written.
    #[must_use]
    pub const fn is_writable(&self) -> bool {
        self.is_field() || self.is_setter()
    }
    /// Returns whether the property is getter-only and computed.
    #[must_use]
    pub const fn is_computed(&self) -> bool {
        !self.is_field() && self.is_getter()
    }

    /// Returns the storage classification.
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
    /// # Errors
    ///
    /// Returns [`PropertyAccessError::NotReadable`] when neither a getter nor
    /// a backing field exists, and otherwise propagates access failures.
    #[must_use = "handle property access failure"]
    pub fn get<'a>(&self, target: ReflectedRef<'a>) -> Result<PropertyValue<'a>, PropertyAccessError> {
        if let Some(getter) = self.getter {
            return getter.get(target);
        }
        if let Some(field) = self.field {
            return field
                .reflect()
                .expect("runtime properties only reference concrete fields")
                .get(target)
                .map(PropertyValue::Borrowed)
                .map_err(Into::into);
        }
        Err(PropertyAccessError::NotReadable)
    }

    /// Writes with explicit setter precedence over field fallback.
    ///
    /// # Errors
    ///
    /// Returns [`PropertySetFailure`] retaining the replacement when no write
    /// operation has started, and otherwise reports the setter or field error.
    #[must_use = "handle property write failure and recover the replacement when available"]
    pub fn set(&self, target: ReflectedMut<'_>, value: ReflectedOwned) -> Result<(), PropertySetFailure> {
        if let Some(setter) = self.setter {
            return setter.set(target, value);
        }
        if let Some(field) = self.field {
            return field
                .reflect()
                .expect("runtime properties only reference concrete fields")
                .set(target, value)
                .map_err(|failure| {
                    let (error, recovery) = failure.into_parts();
                    PropertySetFailure {
                        error: Box::new(PropertyAccessError::Field(error)),
                        replacement: recovery.map(FieldSetRecovery::into_value).map(Box::new),
                    }
                });
        }
        Err(PropertySetFailure::before_execution(
            PropertyAccessError::NotWritable,
            value,
        ))
    }
}

/// Returns the concrete type ID represented by a reflected shared borrow.
fn reflected_ref_type_id(value: &ReflectedRef<'_>) -> TypeId {
    value.as_any().map_or_else(TypeId::of::<str>, std::any::Any::type_id)
}

/// Returns the concrete type ID represented by a reflected mutable borrow.
fn reflected_mut_type_id(value: &ReflectedMut<'_>) -> TypeId {
    value.as_any().map_or_else(TypeId::of::<str>, std::any::Any::type_id)
}

/// Returns the concrete type ID represented by an owned reflected value.
fn reflected_owned_type_id(value: &ReflectedOwned) -> TypeId {
    value.as_any().map_or_else(TypeId::of::<()>, std::any::Any::type_id)
}
