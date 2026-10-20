// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Safe local erased adapters for model properties.

use std::any::TypeId;
use std::error::Error;
use std::fmt;

use qubit_reflect::__private::codegen_v4::expression::TypeExpressionData;
use qubit_reflect::InvocationOutput;
use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::BorrowOrigin;
use qubit_reflect::descriptor::FieldAccessError;
use qubit_reflect::descriptor::FieldSetError;
use qubit_reflect::descriptor::TypeExpression;
use qubit_reflect::value::DynamicMut;
use qubit_reflect::value::DynamicOwned;
use qubit_reflect::value::DynamicRef;
use qubit_reflect::value::Local;
use thiserror::Error;

use crate::metadata::FieldMetadata;
#[cfg(feature = "validation")]
use crate::type_metadata::TypeMetadata;

/// Local erased shared borrow owned by the metadata layer.
pub type PropertyBorrowed<'a> = DynamicRef<'a, Local>;

/// Local erased mutable borrow owned by the metadata layer.
pub type PropertyMutable<'a> = DynamicMut<'a, Local>;

/// Local erased owned value owned by the metadata layer.
pub type PropertyOwned = DynamicOwned<Local>;

/// Exact Rust identities involved in a rejected property operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PropertyTypeMismatch {
    /// Expected Rust type identity.
    pub expected: TypeId,
    /// Actual Rust type identity.
    pub actual: TypeId,
}

/// Generated exact type operations for traversing an `Option<T>` property.
#[derive(Clone, Copy)]
pub struct OptionalOps {
    element_type_id: fn() -> TypeId,
    project_ref: for<'a> fn(PropertyBorrowed<'a>) -> Result<Option<PropertyBorrowed<'a>>, PropertyTypeMismatch>,
    project_mut: for<'a> fn(PropertyMutable<'a>) -> Result<Option<PropertyMutable<'a>>, PropertyTypeMismatch>,
}

impl OptionalOps {
    /// Builds projection operations for one reflected element type.
    #[doc(hidden)]
    pub fn for_element<
        T: qubit_reflect::descriptor::Reflect
            + qubit_reflect::value::ValueForRef<Local>
            + qubit_reflect::value::ValueForMut<Local>
            + std::any::Any,
    >() -> Self {
        fn type_id<T: 'static>() -> TypeId {
            TypeId::of::<T>()
        }
        fn project_ref<'a, T>(value: PropertyBorrowed<'a>) -> Result<Option<PropertyBorrowed<'a>>, PropertyTypeMismatch>
        where
            T: qubit_reflect::descriptor::Reflect + qubit_reflect::value::ValueForRef<Local> + std::any::Any,
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
        fn project_mut<'a, T>(value: PropertyMutable<'a>) -> Result<Option<PropertyMutable<'a>>, PropertyTypeMismatch>
        where
            T: qubit_reflect::descriptor::Reflect + qubit_reflect::value::ValueForMut<Local>,
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
    pub fn element_type_id(self) -> TypeId {
        (self.element_type_id)()
    }

    /// Projects `Some(T)` as a borrow, returning `None` for an absent value.
    pub fn project_ref<'a>(
        self,
        value: PropertyBorrowed<'a>,
    ) -> Result<Option<PropertyBorrowed<'a>>, PropertyTypeMismatch> {
        (self.project_ref)(value)
    }

    /// Projects `Some(T)` as a mutable borrow, returning `None` if absent.
    pub fn project_mut<'a>(
        self,
        value: PropertyMutable<'a>,
    ) -> Result<Option<PropertyMutable<'a>>, PropertyTypeMismatch> {
        (self.project_mut)(value)
    }
}

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
    /// No reflected field or explicit getter provides the value.
    Virtual,
}

/// A property getter result that either borrows from its target or owns a
/// value.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyOwned;
/// use qubit_model_metadata::metadata::PropertyValue;
///
///
/// let value = PropertyValue::Owned(PropertyOwned::new(42_u32));
/// assert!(matches!(value, PropertyValue::Owned(_)));
/// ```
#[must_use]
pub enum PropertyValue<'a> {
    /// A dynamically typed borrow tied to the target.
    Borrowed(PropertyBorrowed<'a>),
    /// An optional dynamically typed borrow.
    OptionalBorrowed(Option<PropertyBorrowed<'a>>),
    /// A lifetime-preserving borrowed slice.
    BorrowedSlice(BorrowedPropertySlice<'a>),
    /// An owned dynamically typed value.
    Owned(PropertyOwned),
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
    #[must_use = "handle the reflected invocation output"]
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
    fn get(&self, index: usize) -> Option<PropertyBorrowed<'a>>;
    /// Iterates over borrowed elements without cloning them.
    fn iter(&self) -> Box<dyn Iterator<Item = PropertyBorrowed<'a>> + 'a>;
}

/// Implements [`PropertySlice`] for a slice with one concrete element type.
struct TypedPropertySlice<'a, T> {
    /// The typed slice retained by the erased adapter.
    values: &'a [T],
}

impl<'a, T: qubit_reflect::value::ValueForRef<Local>> PropertySlice<'a> for TypedPropertySlice<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    fn get(&self, index: usize) -> Option<PropertyBorrowed<'a>> {
        self.values.get(index).map(PropertyBorrowed::new)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = PropertyBorrowed<'a>> + 'a> {
        Box::new(self.values.iter().map(PropertyBorrowed::new))
    }
}

/// A lifetime-preserving, type-erased borrowed slice returned by a property.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::BorrowedPropertySlice;
///
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
    pub fn new<T: qubit_reflect::value::ValueForRef<Local>>(value: &'a [T]) -> Self {
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
    pub fn get(&self, index: usize) -> Option<PropertyBorrowed<'a>> {
        self.value.get(index)
    }

    /// Returns a borrowing iterator over every element.
    #[must_use]
    pub fn iter(&self) -> Box<dyn Iterator<Item = PropertyBorrowed<'a>> + 'a> {
        self.value.iter()
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
/// use qubit_reflect::value::Local;
/// use qubit_model_metadata::metadata::GetterAdapter;
/// use qubit_model_metadata::metadata::PropertyAccessError;
/// use qubit_model_metadata::metadata::PropertyBorrowed;
/// use qubit_model_metadata::metadata::PropertyOwned;
/// use qubit_model_metadata::metadata::PropertyValue;
///
///
///
/// fn get_count<'a>(_: PropertyBorrowed<'a>) -> Result<PropertyValue<'a>, PropertyAccessError> {
///     Ok(PropertyValue::Owned(PropertyOwned::new(3_u32)))
/// }
///
/// let adapter: GetterAdapter = get_count;
/// assert!(adapter(PropertyBorrowed::new(&())).is_ok());
/// ```
pub type GetterAdapter = for<'a> fn(PropertyBorrowed<'a>) -> Result<PropertyValue<'a>, PropertyAccessError>;

/// Reads a map length from an exact borrowed property value; `None` means an
/// optional map field is absent.
///
/// The adapter returns `Ok(None)` when the property value is an absent optional
/// map, and `Ok(Some(length))` for a present map.
///
/// # Examples
///
/// ```
/// use qubit_reflect::value::Local;
/// use qubit_model_metadata::metadata::PropertyAccessError;
/// use qubit_model_metadata::metadata::PropertyOwned;
/// use qubit_model_metadata::metadata::PropertyValue;
///
/// fn map_len(_: &PropertyValue<'_>) -> Result<Option<usize>, PropertyAccessError> {
///     Ok(Some(2))
/// }
///
/// let value = PropertyValue::Owned(PropertyOwned::new(()));
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
/// use qubit_reflect::value::Local;
/// use qubit_model_metadata::metadata::PropertyAccessError;
/// use qubit_model_metadata::metadata::PropertyBorrowed;
///
///
/// fn equal(_: PropertyBorrowed<'_>, _: PropertyBorrowed<'_>) -> Result<bool, PropertyAccessError> {
///     Ok(true)
/// }
///
/// assert!(equal(PropertyBorrowed::new(&1_u32), PropertyBorrowed::new(&1_u32)).expect("comparison"));
/// ```
pub type ItemEqAdapter = for<'a> fn(PropertyBorrowed<'a>, PropertyBorrowed<'a>) -> Result<bool, PropertyAccessError>;

/// A borrowed iterator over members selected from one collection value.
#[must_use]
pub struct BorrowedCollection<'a> {
    values: Box<dyn Iterator<Item = PropertyBorrowed<'a>> + 'a>,
}

impl<'a> BorrowedCollection<'a> {
    /// Erases an iterator while retaining the lifetime of its borrowed items.
    #[doc(hidden)]
    pub fn new(values: impl Iterator<Item = PropertyBorrowed<'a>> + 'a) -> Self {
        Self {
            values: Box::new(values),
        }
    }

    /// Returns the iterator's exact remaining length when it is known.
    #[must_use]
    pub fn remaining_len(&self) -> Option<usize> {
        let (lower, upper) = self.values.size_hint();
        (upper == Some(lower)).then_some(lower)
    }
}

impl<'a> Iterator for BorrowedCollection<'a> {
    type Item = PropertyBorrowed<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.values.next()
    }
}

/// Reads selected members without extending their borrow lifetime.
pub type CollectionItemsAdapter = for<'a> fn(
    &PropertyValue<'a>,
    crate::metadata::SelectorPosition,
) -> Result<Option<BorrowedCollection<'a>>, PropertyAccessError>;

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
    collection_items: Option<CollectionItemsAdapter>,
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
        Self {
            map_len,
            item_eq,
            collection_items: None,
        }
    }

    /// Adds the exact-type borrowing adapter used by executable selectors.
    #[doc(hidden)]
    #[must_use]
    pub const fn with_items(mut self, collection_items: CollectionItemsAdapter) -> Self {
        self.collection_items = Some(collection_items);
        self
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

    /// Returns the exact-type borrowing adapter for sequence or map members.
    #[must_use]
    #[inline]
    pub const fn collection_items(&self) -> Option<CollectionItemsAdapter> {
        self.collection_items
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
/// use qubit_reflect::value::Local;
/// use qubit_model_metadata::metadata::PropertySetFailure;
/// use qubit_model_metadata::metadata::PropertyMutable;
/// use qubit_model_metadata::metadata::PropertyOwned;
/// use qubit_model_metadata::metadata::SetterAdapter;
///
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
    TargetTypeMismatch(PropertyTypeMismatch),
    /// The replacement has a different concrete type.
    #[error("property value type mismatch")]
    ValueTypeMismatch(PropertyTypeMismatch),
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
    /// The underlying reflection field replacement failed.
    #[error("reflected field replacement failed: {0}")]
    FieldSet(#[from] FieldSetError),
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
    pub fn value_type_mismatch<T: 'static>(actual: PropertyBorrowed<'_>) -> Self {
        Self::ValueTypeMismatch(PropertyTypeMismatch {
            expected: TypeId::of::<T>(),
            actual: actual.type_id(),
        })
    }
}

/// A property set failure with optional untouched replacement ownership.
///
/// # Examples
///
/// ```
/// use qubit_reflect::value::Local;
/// use qubit_model_metadata::metadata::PropertyAccessError;
/// use qubit_model_metadata::metadata::PropertySetFailure;
/// use qubit_model_metadata::metadata::PropertyOwned;
///
///
/// let failure = PropertySetFailure::before_execution(
///     PropertyAccessError::NotWritable,
///     PropertyOwned::new(7_u32),
/// );
/// assert!(failure.replacement().is_some());
/// ```
#[must_use]
pub struct PropertySetFailure {
    /// The structured reason why the property write failed.
    error: Box<PropertyAccessError>,
    /// The untouched replacement retained for a pre-execution failure.
    replacement: Option<Box<PropertyOwned>>,
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
    pub fn before_execution(error: PropertyAccessError, replacement: PropertyOwned) -> Self {
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
    pub fn replacement(&self) -> Option<&PropertyOwned> {
        self.replacement.as_deref()
    }

    /// Consumes the failure and returns its parts.
    ///
    /// # Returns
    ///
    /// The underlying access error and, when available, the untouched
    /// replacement.
    #[must_use = "handle the property error and recovered replacement"]
    pub fn into_parts(self) -> (PropertyAccessError, Option<PropertyOwned>) {
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
/// use qubit_reflect::value::Local;
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
///
///
/// #[Model]
/// pub struct Item { pub value: u32 }
///
/// pub fn get_value<'a>(target: PropertyBorrowed<'a>) -> Result<PropertyValue<'a>, PropertyAccessError> {
///     let item = target.downcast::<Item>().ok().expect("validated Item target");
///     Ok(PropertyValue::Borrowed(PropertyBorrowed::new(&item.value)))
/// }
/// # }
///
/// let value_type = qubit_model_metadata::metadata::TypeMetadata::of::<example::Item>()
///     .field("value").expect("value field").type_expression_data().expect("field type");
/// let getter = GetterMetadata::new::<example::Item>("value", value_type,
///     std::any::TypeId::of::<u32>, GetterOutputKind::Borrowed, example::get_value);
/// assert_eq!(getter.rust_method_name(), "value");
/// ```
#[derive(Clone, Copy)]
pub struct GetterMetadata {
    /// The Rust identifier of the generated getter method.
    rust_method_name: &'static str,
    /// The declared type of the getter output.
    output_type: &'static TypeExpressionData,
    output_type_id: fn() -> TypeId,
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

    /// Binds this getter's declared output expression to `context`.
    pub fn output_type_in<'ctx>(&self, context: &'ctx ReflectionContext) -> TypeExpression<'ctx> {
        qubit_reflect::__private::codegen_v4::expression::type_expression_in(self.output_type, context)
    }

    /// Returns the process-local identity of the declared output type.
    pub fn output_type_id(&self) -> TypeId {
        (self.output_type_id)()
    }

    /// Returns the exact element identity for a borrowed-slice getter.
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
    #[must_use = "handle property access failure"]
    pub fn get<'a, T: qubit_reflect::value::ValueForRef<Local>>(
        &self,
        target: &'a T,
    ) -> Result<PropertyValue<'a>, PropertyAccessError> {
        self.get_dynamic(PropertyBorrowed::new(target))
    }

    /// Executes this getter using a dynamically borrowed target.
    ///
    /// Returns [`PropertyAccessError::TargetTypeMismatch`] when the target's
    /// concrete type differs from the getter's declared type, or propagates
    /// the generated adapter error. The result preserves its borrowed or
    /// owned representation.
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
    pub fn input_type_in<'ctx>(&self, context: &'ctx ReflectionContext) -> TypeExpression<'ctx> {
        qubit_reflect::__private::codegen_v4::expression::type_expression_in(self.input_type, context)
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
    pub fn set<T: qubit_reflect::value::ValueForMut<Local>, V: qubit_reflect::value::ValueFor<Local>>(
        &self,
        target: &mut T,
        value: V,
    ) -> Result<(), PropertySetFailure> {
        self.set_dynamic(PropertyMutable::new(target), PropertyOwned::new(value))
    }

    /// Executes this setter using a dynamic target and owned replacement.
    ///
    /// Returns [`PropertySetFailure`] with the replacement retained when
    /// target or input validation fails before adapter execution; adapter
    /// failures carry the recovery state produced by the setter.
    pub fn set_dynamic(&self, target: PropertyMutable<'_>, value: PropertyOwned) -> Result<(), PropertySetFailure> {
        let actual_target = target.type_id();
        let expected_target = (self.target_type_id)();
        if actual_target != expected_target {
            return Err(PropertySetFailure::before_execution(
                PropertyAccessError::TargetTypeMismatch(PropertyTypeMismatch {
                    expected: expected_target,
                    actual: actual_target,
                }),
                value,
            ));
        }
        let actual_value = value.type_id();
        let expected_value = (self.input_type_id)();
        if actual_value != expected_value {
            return Err(PropertySetFailure::before_execution(
                PropertyAccessError::ValueTypeMismatch(PropertyTypeMismatch {
                    expected: expected_value,
                    actual: actual_value,
                }),
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
/// #[Model(id = "example.PropertyMetadataNote")]
/// pub struct Note { pub title: String }
/// # }
///
/// let metadata = TypeMetadata::of::<example::Note>();
/// let properties = metadata.try_properties().expect("property metadata");
/// let title = properties.property("title").expect("title property");
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
    /// Returns the retained type-expression data for generated metadata.
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

    /// Returns this property's type expression bound to `context`.
    ///
    /// A backing field determines the expression for field-backed properties.
    /// Otherwise the explicit declaration or accessor signature is used.
    #[must_use]
    #[inline]
    pub fn type_expression_in<'ctx>(&self, context: &'ctx ReflectionContext) -> Option<TypeExpression<'ctx>> {
        if let Some(field) = self.field
            && let Some(expression) = field.type_expression_in(context)
        {
            return Some(expression);
        }
        if let Some(expression) = self.type_expression {
            return Some(qubit_reflect::__private::codegen_v4::expression::type_expression_in(
                expression, context,
            ));
        }
        if let Some(getter) = self.getter {
            return Some(getter.output_type_in(context));
        }
        self.setter.map(|setter| setter.input_type_in(context))
    }

    pub(crate) fn declared_type_expression_in<'ctx>(
        &self,
        context: &'ctx ReflectionContext,
    ) -> Option<TypeExpression<'ctx>> {
        self.type_expression
            .map(|expression| qubit_reflect::__private::codegen_v4::expression::type_expression_in(expression, context))
    }

    /// Resolves this property's concrete type in `context`, when available.
    ///
    /// # Returns
    ///
    /// `Some` with the concrete descriptor when resolved, or `None` for
    /// symbolic and opaque types.
    #[inline]
    pub fn descriptor_in<'ctx>(
        &self,
        context: &'ctx ReflectionContext,
    ) -> Result<Option<TypeDescriptor<'ctx>>, qubit_reflect::error::ContextError> {
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

    /// Resolves a field-backed property's descriptor from its known owner.
    #[cfg(feature = "validation")]
    pub(crate) fn descriptor_in_owner<'ctx>(
        &self,
        owner: &'static TypeMetadata,
        context: &'ctx ReflectionContext,
    ) -> Result<Option<TypeDescriptor<'ctx>>, qubit_reflect::error::ContextError> {
        if let Some(field) = self.field
            && self.getter.is_none()
            && let Some(reflected) = owner.field_descriptor_in(field, context)
        {
            return Ok(reflected.find_concrete_type());
        }
        self.descriptor_in(context)
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
    /// Returns generated projection operations for an optional backing field.
    #[must_use]
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
    pub fn get<'a, T: qubit_reflect::value::ValueForRef<Local>>(
        &self,
        target: &'a T,
    ) -> Result<PropertyValue<'a>, PropertyAccessError> {
        self.get_dynamic(PropertyBorrowed::new(target))
    }

    /// Reads this property using a dynamically borrowed target.
    ///
    /// A getter takes precedence over a backing field. Returns
    /// [`PropertyAccessError::NotReadable`] when neither is available, and
    /// otherwise propagates access failures while preserving borrowed or
    /// owned output.
    pub fn get_dynamic<'a>(&self, target: PropertyBorrowed<'a>) -> Result<PropertyValue<'a>, PropertyAccessError> {
        self.get_dynamic_in(ReflectionContext::global(), target)
    }

    /// Reads this property using descriptors bound to `context`.
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
    pub fn set<T: qubit_reflect::value::ValueForMut<Local>, V: qubit_reflect::value::ValueFor<Local>>(
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
    pub fn set_dynamic(&self, target: PropertyMutable<'_>, value: PropertyOwned) -> Result<(), PropertySetFailure> {
        self.set_dynamic_in(ReflectionContext::global(), target, value)
    }

    /// Writes using descriptors bound to `context`.
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
                .map_err(|failure| {
                    let (error, recovery) = failure.into_parts();
                    PropertySetFailure {
                        error: Box::new(PropertyAccessError::FieldSet(error)),
                        replacement: Some(Box::new(recovery)),
                    }
                });
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
            return unsafe { reflected.set_dynamic_unsafe(target, value) }.map_err(|failure| {
                let (error, recovery) = failure.into_parts();
                PropertySetFailure {
                    error: Box::new(PropertyAccessError::FieldSet(error)),
                    replacement: Some(Box::new(recovery)),
                }
            });
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
    pub unsafe fn set_dynamic_unsafe(
        &self,
        target: PropertyMutable<'_>,
        value: PropertyOwned,
    ) -> Result<(), PropertySetFailure> {
        self.set_dynamic(target, value)
    }
}
