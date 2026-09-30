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
use std::error::Error;
use std::fmt;

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
use thiserror::Error;

use crate::metadata::FieldMetadata;

/// Classifies how a property stores or computes its value.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyStorageKind;
///
/// let kind = PropertyStorageKind::Computed;
/// assert_eq!(kind, PropertyStorageKind::Computed);
/// ```
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
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyValue;
/// use qubit_reflect::ReflectedOwned;
///
/// let value = PropertyValue::Owned(ReflectedOwned::new(42_u32));
/// assert!(matches!(value, PropertyValue::Owned(_)));
/// ```
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

/// Erases a borrowed slice while preserving the lifetime of each element.
trait PropertySlice<'a> {
    /// Returns the number of values in the borrowed slice.
    fn len(&self) -> usize;
    /// Returns the value at `index`, or `None` when it is out of bounds.
    fn get(&self, index: usize) -> Option<ReflectedRef<'a>>;
}

/// Implements [`PropertySlice`] for a slice with one concrete element type.
struct TypedPropertySlice<'a, T> {
    /// The typed slice retained by the erased adapter.
    values: &'a [T],
}

impl<'a, T: 'static> PropertySlice<'a> for TypedPropertySlice<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    fn get(&self, index: usize) -> Option<ReflectedRef<'a>> {
        self.values.get(index).map(ReflectedRef::new)
    }
}

/// A lifetime-preserving, type-erased borrowed slice returned by a property.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::BorrowedPropertySlice;
/// use qubit_reflect::ReflectedRef;
///
/// let values = [3_u32, 5];
/// let slice = BorrowedPropertySlice::new(&values);
/// assert_eq!(slice.len(), 2);
/// assert!(slice.get(0).is_some());
/// ```
#[must_use]
pub struct BorrowedPropertySlice<'a> {
    /// The lifetime-preserving erased slice implementation.
    value: Box<dyn PropertySlice<'a> + 'a>,
}

impl<'a> BorrowedPropertySlice<'a> {
    /// Erases a borrowed slice without extending its lifetime.
    ///
    /// # Parameters
    ///
    /// * `value` - The slice whose elements remain borrowed for `'a`.
    ///
    /// # Returns
    ///
    /// A type-erased adapter that preserves the input slice's lifetime.
    #[doc(hidden)]
    pub fn new<T: 'static>(value: &'a [T]) -> Self {
        Self {
            value: Box::new(TypedPropertySlice { values: value }),
        }
    }

    /// Returns the number of elements.
    ///
    /// # Returns
    ///
    /// The number of elements in the retained slice.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.value.len()
    }

    /// Returns whether the slice contains no elements.
    ///
    /// # Returns
    ///
    /// `true` when the retained slice has no elements.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns one element as a borrowed reflected value.
    ///
    /// # Parameters
    ///
    /// * `index` - The zero-based element position.
    ///
    /// # Returns
    ///
    /// `Some` with the borrowed element when `index` is in bounds, or `None`
    /// otherwise.
    #[must_use]
    #[inline]
    pub fn get(&self, index: usize) -> Option<ReflectedRef<'a>> {
        self.value.get(index)
    }
}

/// Distinguishes lifetime-preserving borrowed output from owned output.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::GetterOutputKind;
///
/// let output = GetterOutputKind::Borrowed;
/// assert_eq!(output, GetterOutputKind::Borrowed);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GetterOutputKind {
    /// The adapter returns an optional borrow of the contained value.
    OptionalBorrowed,
    /// The adapter returns a slice supporting borrowed element access.
    BorrowedSlice,
    /// The output borrows from its target.
    Borrowed,
    /// The output owns its value.
    Owned,
}

/// A lifetime-preserving local getter adapter.
///
/// The function receives a borrowed target and returns a borrowed or owned
/// erased property value.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::GetterAdapter;
/// use qubit_model_metadata::metadata::PropertyAccessError;
/// use qubit_model_metadata::metadata::PropertyValue;
/// use qubit_reflect::ReflectedRef;
/// use qubit_reflect::ReflectedOwned;
///
/// fn get_count<'a>(_: ReflectedRef<'a>) -> Result<PropertyValue<'a>, PropertyAccessError> {
///     Ok(PropertyValue::Owned(ReflectedOwned::new(3_u32)))
/// }
///
/// let adapter: GetterAdapter = get_count;
/// assert!(adapter(ReflectedRef::new(&())).is_ok());
/// ```
pub type GetterAdapter = for<'a> fn(ReflectedRef<'a>) -> Result<PropertyValue<'a>, PropertyAccessError>;

/// Reads a map length from an exact borrowed property value; `None` means an
/// optional map field is absent.
///
/// The adapter returns `Ok(None)` when the property value is an absent optional
/// map, and `Ok(Some(length))` for a present map.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyAccessError;
/// use qubit_model_metadata::metadata::PropertyValue;
///
/// fn map_len(_: &PropertyValue<'_>) -> Result<Option<usize>, PropertyAccessError> {
///     Ok(Some(2))
/// }
///
/// let value = PropertyValue::Owned(qubit_reflect::ReflectedOwned::new(()));
/// assert_eq!(map_len(&value).expect("map length"), Some(2));
/// ```
pub type MapLenAdapter = for<'a> fn(&PropertyValue<'a>) -> Result<Option<usize>, PropertyAccessError>;

/// Compares two exact borrowed collection elements.
///
/// The adapter reports type or access failures through [`PropertyAccessError`].
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyAccessError;
/// use qubit_reflect::ReflectedRef;
///
/// fn equal(_: ReflectedRef<'_>, _: ReflectedRef<'_>) -> Result<bool, PropertyAccessError> {
///     Ok(true)
/// }
///
/// assert!(equal(ReflectedRef::new(&1_u32), ReflectedRef::new(&1_u32)).expect("comparison"));
/// ```
pub type ItemEqAdapter = for<'a> fn(ReflectedRef<'a>, ReflectedRef<'a>) -> Result<bool, PropertyAccessError>;

/// Safe, read-only operations available for a declared collection field.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::CollectionOps;
///
/// let operations = CollectionOps::new(None, None);
/// assert!(operations.map_len().is_none());
/// ```
#[derive(Clone, Copy, Debug)]
pub struct CollectionOps {
    map_len: Option<MapLenAdapter>,
    item_eq: Option<ItemEqAdapter>,
}

impl CollectionOps {
    /// Creates an operation set. Generated adapters are tied to exact Rust
    /// types.
    ///
    /// # Parameters
    ///
    /// * `map_len` - Optional adapter for reading a map's length.
    /// * `item_eq` - Optional adapter for comparing collection elements.
    ///
    /// # Returns
    ///
    /// The collection operation set containing the supplied adapters.
    #[must_use]
    #[inline]
    pub const fn new(map_len: Option<MapLenAdapter>, item_eq: Option<ItemEqAdapter>) -> Self {
        Self { map_len, item_eq }
    }

    /// Returns the optional map length adapter.
    ///
    /// # Returns
    ///
    /// `Some` when a map-length adapter is configured, or `None` otherwise.
    #[must_use]
    #[inline]
    pub const fn map_len(&self) -> Option<MapLenAdapter> {
        self.map_len
    }

    /// Returns the optional element equality adapter.
    ///
    /// # Returns
    ///
    /// `Some` when an element-equality adapter is configured, or `None`
    /// otherwise.
    #[must_use]
    #[inline]
    pub const fn item_eq(&self) -> Option<ItemEqAdapter> {
        self.item_eq
    }
}

/// A local setter adapter with recoverable pre-execution failure.
///
/// The adapter returns a [`PropertySetFailure`] that can retain the replacement
/// when validation fails before ownership crosses the execution boundary.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertySetFailure;
/// use qubit_model_metadata::metadata::SetterAdapter;
/// use qubit_reflect::ReflectedMut;
/// use qubit_reflect::ReflectedOwned;
///
/// fn set_value(_: ReflectedMut<'_>, _: ReflectedOwned) -> Result<(), PropertySetFailure> {
///     Ok(())
/// }
///
/// let adapter: SetterAdapter = set_value;
/// assert!(adapter(ReflectedMut::new(&mut 1_u32), ReflectedOwned::new(2_u32)).is_ok());
/// ```
pub type SetterAdapter = fn(ReflectedMut<'_>, ReflectedOwned) -> Result<(), PropertySetFailure>;

/// A property operation failed before or during adapter execution.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyAccessError;
///
/// let error = PropertyAccessError::NotReadable;
/// assert!(matches!(error, PropertyAccessError::NotReadable));
/// ```
#[must_use]
#[derive(Debug, Error)]
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
    ///
    /// # Parameters
    ///
    /// * `message` - A static message describing the adapter failure.
    ///
    /// # Returns
    ///
    /// An error carrying the supplied user-facing message.
    #[must_use = "handle the property access error"]
    #[inline]
    pub const fn user(message: &'static str) -> Self {
        Self::User(message)
    }

    /// Reports that an erased borrowed value has the wrong exact Rust type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The expected concrete Rust type.
    ///
    /// # Parameters
    ///
    /// * `actual` - The erased borrowed value whose type was observed.
    ///
    /// # Returns
    ///
    /// A type-mismatch error containing expected and actual type identities.
    #[doc(hidden)]
    pub fn value_type_mismatch<T: 'static>(actual: &ReflectedRef<'_>) -> Self {
        Self::ValueTypeMismatch(TypeMismatch::new(TypeId::of::<T>(), actual.value_type_id()))
    }
}

/// A property set failure with optional untouched replacement ownership.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyAccessError;
/// use qubit_model_metadata::metadata::PropertySetFailure;
/// use qubit_reflect::ReflectedOwned;
///
/// let failure = PropertySetFailure::before_execution(
///     PropertyAccessError::NotWritable,
///     ReflectedOwned::new(7_u32),
/// );
/// assert!(failure.replacement().is_some());
/// ```
#[must_use]
pub struct PropertySetFailure {
    /// The structured reason why the property write failed.
    error: Box<PropertyAccessError>,
    /// The untouched replacement retained for a pre-execution failure.
    replacement: Option<Box<ReflectedOwned>>,
}

impl PropertySetFailure {
    /// Creates a pre-execution failure retaining the replacement.
    ///
    /// # Parameters
    ///
    /// * `error` - The reason the write could not start.
    /// * `replacement` - The owned value that remains available for recovery.
    ///
    /// # Returns
    ///
    /// A failure that preserves ownership of `replacement`.
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
    ///
    /// # Parameters
    ///
    /// * `error` - The reason the adapter failed after accepting the value.
    ///
    /// # Returns
    ///
    /// A failure that records no recoverable replacement.
    #[doc(hidden)]
    #[must_use = "handle the property set failure"]
    pub fn after_execution(error: PropertyAccessError) -> Self {
        Self {
            error: Box::new(error),
            replacement: None,
        }
    }

    /// Returns the structured failure.
    ///
    /// # Returns
    ///
    /// A reference to the underlying property access error.
    #[must_use = "inspect the property failure before discarding it"]
    #[inline]
    pub const fn error(&self) -> &PropertyAccessError {
        &self.error
    }

    /// Returns the untouched replacement for pre-execution failure.
    ///
    /// # Returns
    ///
    /// `Some` with the untouched replacement when execution did not begin, or
    /// `None` after ownership crossed the execution boundary.
    #[must_use]
    #[inline]
    pub fn replacement(&self) -> Option<&ReflectedOwned> {
        self.replacement.as_deref()
    }

    /// Consumes the failure and returns its parts.
    ///
    /// # Returns
    ///
    /// The underlying access error and, when available, the untouched
    /// replacement.
    #[must_use = "handle the property error and recovered replacement"]
    pub fn into_parts(self) -> (PropertyAccessError, Option<ReflectedOwned>) {
        (*self.error, self.replacement.map(|replacement| *replacement))
    }
}

impl fmt::Debug for PropertySetFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PropertySetFailure")
            .field("error", &self.error)
            .field("has_replacement", &self.replacement.is_some())
            .finish()
    }
}

impl fmt::Display for PropertySetFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(formatter)
    }
}

impl Error for PropertySetFailure {}

/// Metadata and adapter for one explicit getter method.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::GetterMetadata;
/// use qubit_model_metadata::metadata::GetterOutputKind;
/// use qubit_model_metadata::metadata::PropertyAccessError;
/// use qubit_model_metadata::metadata::PropertyValue;
/// use qubit_reflect::Reflect;
/// use qubit_reflect::ReflectedRef;
/// use qubit_reflect::TypeDescriptor;
///
/// # mod example {
/// use qubit_model_metadata::metadata::PropertyAccessError;
/// use qubit_model_metadata::metadata::PropertyValue;
/// use qubit_reflect::Reflect;
/// use qubit_reflect::ReflectedRef;
///
/// #[derive(Reflect)]
/// pub struct Item { pub value: u32 }
///
/// pub fn get_value<'a>(target: ReflectedRef<'a>) -> Result<PropertyValue<'a>, PropertyAccessError> {
///     let item = target.downcast::<Item>().ok().expect("validated Item target");
///     Ok(PropertyValue::Borrowed(ReflectedRef::new(&item.value)))
/// }
/// # }
///
/// let descriptor = TypeDescriptor::of::<example::Item>();
/// let value_type = descriptor.field_at(0).expect("value field").field_type();
/// let getter = GetterMetadata::new::<example::Item>("value", value_type, GetterOutputKind::Borrowed, example::get_value);
/// assert_eq!(getter.rust_method_name(), "value");
/// ```
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
    ///
    /// # Type Parameters
    ///
    /// * `Target` - The exact Rust target type accepted by the getter.
    ///
    /// # Parameters
    ///
    /// * `rust_method_name` - The source-level getter method name.
    /// * `output_type` - The declared reflection type of the getter result.
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
    pub const fn output_type(&self) -> &'static TypeRef {
        self.output_type
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
    #[must_use = "handle property access failure"]
    pub fn get<'a>(&self, target: ReflectedRef<'a>) -> Result<PropertyValue<'a>, PropertyAccessError> {
        let actual = target.value_type_id();
        let expected = (self.target_type_id)();
        if actual != expected {
            return Err(PropertyAccessError::TargetTypeMismatch(TypeMismatch::new(
                expected, actual,
            )));
        }
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

/// Metadata and adapter for one explicit setter method.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::SetterMetadata;
/// use qubit_reflect::Reflect;
/// use qubit_reflect::ReflectedMut;
/// use qubit_reflect::ReflectedOwned;
/// use qubit_reflect::TypeDescriptor;
///
/// # mod example {
/// use qubit_model_metadata::metadata::PropertySetFailure;
/// use qubit_reflect::Reflect;
/// use qubit_reflect::ReflectedMut;
/// use qubit_reflect::ReflectedOwned;
///
/// #[derive(Reflect)]
/// pub struct Item { pub value: u32 }
///
/// pub fn set_value(target: ReflectedMut<'_>, value: ReflectedOwned) -> Result<(), PropertySetFailure> {
///     let mut item = target.downcast::<Item>().ok().expect("validated Item target");
///     item.value = value.downcast::<u32>().ok().expect("validated u32 value");
///     Ok(())
/// }
/// # }
///
/// let descriptor = TypeDescriptor::of::<example::Item>();
/// let value_type = descriptor.field_at(0).expect("value field").field_type();
/// let setter = SetterMetadata::new::<example::Item, u32>("set_value", value_type, example::set_value);
/// assert_eq!(setter.rust_method_name(), "set_value");
/// ```
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
    pub const fn input_type(&self) -> &'static TypeRef {
        self.input_type
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
    #[must_use = "handle property write failure and recover the replacement when available"]
    pub fn set(&self, target: ReflectedMut<'_>, value: ReflectedOwned) -> Result<(), PropertySetFailure> {
        let actual_target = target.value_type_id();
        let expected_target = (self.target_type_id)();
        if actual_target != expected_target {
            return Err(PropertySetFailure::before_execution(
                PropertyAccessError::TargetTypeMismatch(TypeMismatch::new(expected_target, actual_target)),
                value,
            ));
        }
        let actual_value = value.value_type_id();
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

impl fmt::Debug for SetterMetadata {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SetterMetadata")
            .field("rust_method_name", &self.rust_method_name)
            .finish_non_exhaustive()
    }
}

/// Merged field/getter/setter metadata for one model property.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// # mod example {
/// use qubit_model_derive::Model;
///
/// #[Model(id = "example.Note")]
/// pub struct Note { pub title: String }
/// # }
///
/// let metadata = TypeMetadata::of::<example::Note>();
/// let properties = metadata.try_properties().expect("property metadata");
/// let title = properties.property("title").expect("title property");
/// assert_eq!(title.name(), "title");
/// ```
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
    ///
    /// # Parameters
    ///
    /// * `name` - The property's public metadata name.
    /// * `type_ref` - The declared property type.
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
    ///
    /// # Returns
    ///
    /// The property name used by model metadata.
    #[must_use]
    #[inline]
    pub const fn name(&self) -> &'static str {
        self.name
    }
    /// Returns the property type reference.
    ///
    /// # Returns
    ///
    /// The declared type reference, whether resolved or symbolic.
    #[must_use]
    #[inline]
    pub const fn type_ref(&self) -> &'static TypeRef {
        self.type_ref
    }
    /// Returns the resolved property type descriptor, or `None` for symbolic
    /// and opaque property types.
    ///
    /// # Returns
    ///
    /// `Some` with the concrete descriptor when resolved, or `None` for
    /// symbolic and opaque types.
    #[must_use]
    #[inline]
    pub const fn descriptor(&self) -> Option<&'static TypeDescriptor> {
        self.type_ref.as_resolved()
    }
    /// Returns the backing field, or `None` for computed and virtual
    /// properties.
    ///
    /// # Returns
    ///
    /// `Some` with the reflected backing field, or `None` for computed and
    /// virtual properties.
    #[must_use]
    #[inline]
    pub const fn field(&self) -> Option<&'static FieldMetadata> {
        self.field
    }
    /// Returns the explicit getter, or `None` when reads use field fallback.
    ///
    /// # Returns
    ///
    /// `Some` with the explicit getter, or `None` when reads fall back to the
    /// backing field.
    #[must_use]
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
    #[must_use]
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
