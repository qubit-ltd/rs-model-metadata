// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;
use std::fmt;

use qubit_reflect::descriptor::FieldSetError;

use super::PropertyAccessError;
use super::PropertyOwned;

/// A property set failure with optional untouched replacement ownership.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyAccessError;
/// use qubit_model_metadata::metadata::PropertySetFailure;
/// use qubit_model_metadata::metadata::PropertyOwned;
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
    pub fn after_execution(error: PropertyAccessError) -> Self {
        Self {
            error: Box::new(error),
            replacement: None,
        }
    }

    /// Builds a field-write failure while preserving the replacement returned
    /// by reflection after a rejected write.
    pub(super) fn from_field_set_failure(error: FieldSetError, replacement: PropertyOwned) -> Self {
        Self {
            error: Box::new(PropertyAccessError::FieldSet(error)),
            replacement: Some(Box::new(replacement)),
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
