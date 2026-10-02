// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Errors returned while writing through compiled property access paths.

use std::error::Error;
use std::fmt;

use qubit_reflect::ReflectedOwned;

use super::property_access_path_error::PropertyAccessPathError;
use crate::metadata::PropertySetFailure;

/// A path write failure that preserves any replacement still owned by caller.
#[must_use]
pub enum PropertyAccessWriteFailure {
    /// Path validation or intermediate access failed before leaf mutation.
    Path {
        /// Structured path or traversal error.
        error: PropertyAccessPathError,
        /// Replacement retained because the leaf setter did not start.
        replacement: Option<ReflectedOwned>,
    },
    /// The leaf setter or field operation failed with its recovery semantics.
    Property(PropertySetFailure),
}

impl PropertyAccessWriteFailure {
    /// Creates a path failure before the replacement is passed to a setter.
    ///
    /// # Parameters
    ///
    /// - `error`: Path validation or intermediate traversal error.
    /// - `replacement`: Owned value that remains untouched.
    ///
    /// # Returns
    ///
    /// A failure that retains the replacement for recovery.
    pub(crate) fn before_path(error: PropertyAccessPathError, replacement: ReflectedOwned) -> Self {
        Self::Path {
            error,
            replacement: Some(replacement),
        }
    }

    /// Wraps a leaf property failure without changing its recovery state.
    ///
    /// # Parameters
    ///
    /// - `failure`: Original property setter failure.
    ///
    /// # Returns
    ///
    /// The failure with the original error and replacement recovery semantics.
    pub(crate) const fn from_property_set(failure: PropertySetFailure) -> Self {
        Self::Property(failure)
    }

    /// Returns the recoverable replacement, when failure occurred before use.
    ///
    /// # Returns
    ///
    /// The untouched replacement for pre-execution failures, or `None` when
    /// the setter consumed it.
    #[must_use]
    pub fn replacement(&self) -> Option<&ReflectedOwned> {
        match self {
            Self::Path { replacement, .. } => replacement.as_ref(),
            Self::Property(failure) => failure.replacement(),
        }
    }

    /// Returns the path error when traversal failed before reaching the leaf.
    #[must_use]
    pub const fn path_error(&self) -> Option<&PropertyAccessPathError> {
        match self {
            Self::Path { error, .. } => Some(error),
            Self::Property(_) => None,
        }
    }

    /// Returns the leaf property failure when its setter was invoked.
    #[must_use]
    pub const fn property_failure(&self) -> Option<&PropertySetFailure> {
        match self {
            Self::Path { .. } => None,
            Self::Property(failure) => Some(failure),
        }
    }
}

impl fmt::Debug for PropertyAccessWriteFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path { error, replacement } => formatter
                .debug_struct("Path")
                .field("error", error)
                .field("has_replacement", &replacement.is_some())
                .finish(),
            Self::Property(failure) => formatter.debug_tuple("Property").field(failure).finish(),
        }
    }
}

impl fmt::Display for PropertyAccessWriteFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path { error, .. } => error.fmt(formatter),
            Self::Property(failure) => failure.fmt(formatter),
        }
    }
}

impl Error for PropertyAccessWriteFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Path { error, .. } => Some(error),
            Self::Property(failure) => Some(failure),
        }
    }
}
