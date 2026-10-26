// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::any::TypeId;

use qubit_reflect::descriptor::FieldAccessError;
use qubit_reflect::descriptor::FieldSetError;
use thiserror::Error;

use super::PropertyBorrowed;
use super::PropertyTypeMismatch;

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
    #[inline]
    #[doc(hidden)]
    pub fn value_type_mismatch<T: 'static>(actual: PropertyBorrowed<'_>) -> Self {
        Self::ValueTypeMismatch(PropertyTypeMismatch {
            expected: TypeId::of::<T>(),
            actual: actual.type_id(),
        })
    }
}
