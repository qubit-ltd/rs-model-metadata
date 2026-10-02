// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
// =============================================================================

//! Structured errors returned by compiled property access paths.

use std::any::TypeId;
use std::error::Error;
use std::fmt;

use crate::metadata::ModelMetadataError;
use crate::metadata::PropertyAccessError;
use crate::metadata::PropertyResolutionError;
use crate::metadata::PropertySetFailure;
use qubit_reflect::ReflectedOwned;

/// A failure to compile or execute a dynamic property path.
#[derive(Debug, thiserror::Error)]
pub enum PropertyAccessPathError {
    /// No property segments were supplied.
    #[error("property access path is empty")]
    EmptyPath,
    /// A supplied property name is empty.
    #[error("property access path segment {index} is empty")]
    EmptySegment { index: usize },
    /// A property name is absent from the current model.
    #[error("unknown property `{name}` at path segment {index}")]
    UnknownProperty { index: usize, name: String },
    /// A property has no readable field or getter.
    #[error("property `{name}` at path segment {index} is not readable")]
    UnreadableIntermediate { index: usize, name: String },
    /// An intermediate representation cannot safely produce a child borrow.
    #[error("property `{name}` at path segment {index} has an unsupported intermediate type")]
    UnsupportedIntermediate { index: usize, name: String },
    /// An optional intermediate property contains no value.
    #[error("optional property `{name}` at path segment {index} is absent")]
    MissingIntermediate { index: usize, name: String },
    /// An intermediate property cannot be projected mutably.
    #[error("property `{name}` at path segment {index} is not a mutable field")]
    UnwritableIntermediate { index: usize, name: String },
    /// The leaf property has no setter or mutable field.
    #[error("property `{name}` at path segment {index} is not writable")]
    NotWritable { index: usize, name: String },
    /// The supplied reflected root value has a different concrete type.
    #[error("property path root type mismatch: expected {expected:?}, got {actual:?}")]
    RootTypeMismatch { expected: TypeId, actual: TypeId },
    /// Resolving the owner's property set failed.
    #[error("property resolution failed at path segment {index}: {source}")]
    PropertyResolution {
        index: usize,
        #[source]
        source: PropertyResolutionError,
    },
    /// Looking up metadata for a child descriptor failed.
    #[error("model metadata lookup failed at path segment {index}: {source}")]
    MetadataLookup {
        index: usize,
        #[source]
        source: ModelMetadataError,
    },
    /// Reading or mutably projecting a property failed at runtime.
    #[error("property access failed for `{name}` at path segment {index}: {source}")]
    AccessFailure {
        index: usize,
        name: String,
        #[source]
        source: PropertyAccessError,
    },
}

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
