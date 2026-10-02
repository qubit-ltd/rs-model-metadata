// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structured errors returned by compiled property access paths.

use std::any::TypeId;

use crate::metadata::ModelMetadataError;
use crate::metadata::PropertyAccessError;
use crate::metadata::PropertyResolutionError;

/// A failure to compile or execute a dynamic property path.
#[derive(Debug, thiserror::Error)]
pub enum PropertyAccessPathError {
    /// No property segments were supplied.
    #[error("property access path is empty")]
    EmptyPath,
    /// A supplied property name is empty.
    #[error("property access path segment {index} is empty")]
    EmptySegment {
        /// Zero-based path position.
        index: usize,
    },
    /// A property name is absent from the current model.
    #[error("unknown property `{name}` at path segment {index}")]
    UnknownProperty {
        /// Zero-based path position.
        index: usize,
        /// Property name that was not found.
        name: String,
    },
    /// A property has no readable field or getter.
    #[error("property `{name}` at path segment {index} is not readable")]
    UnreadableIntermediate {
        /// Zero-based path position.
        index: usize,
        /// Property name that cannot be read.
        name: String,
    },
    /// An intermediate representation cannot safely produce a child borrow.
    #[error("property `{name}` at path segment {index} has an unsupported intermediate type")]
    UnsupportedIntermediate {
        /// Zero-based path position.
        index: usize,
        /// Property name whose representation cannot be traversed.
        name: String,
    },
    /// An optional intermediate property contains no value.
    #[error("optional property `{name}` at path segment {index} is absent")]
    MissingIntermediate {
        /// Zero-based path position.
        index: usize,
        /// Optional property name with no value.
        name: String,
    },
    /// An intermediate property cannot be projected mutably.
    #[error("property `{name}` at path segment {index} is not a mutable field")]
    UnwritableIntermediate {
        /// Zero-based path position.
        index: usize,
        /// Property name that cannot be mutably projected.
        name: String,
    },
    /// Field metadata has no concrete reflection access descriptor.
    #[error("property `{name}` at path segment {index} has no reflection access adapter")]
    AdapterUnavailable {
        /// Zero-based path position.
        index: usize,
        /// Property name with a missing adapter.
        name: String,
    },
    /// The leaf property has no setter or mutable field.
    #[error("property `{name}` at path segment {index} is not writable")]
    NotWritable {
        /// Zero-based path position.
        index: usize,
        /// Leaf property name without a setter destination.
        name: String,
    },
    /// The supplied reflected root value has a different concrete type.
    #[error("property path root type mismatch: expected {expected:?}, got {actual:?}")]
    RootTypeMismatch {
        /// Expected Rust type identity.
        expected: TypeId,
        /// Actual Rust type identity.
        actual: TypeId,
    },
    /// Resolving the owner's property set failed.
    #[error("property resolution failed at path segment {index}: {source}")]
    PropertyResolution {
        /// Zero-based path position.
        index: usize,
        #[source]
        /// Property resolution failure for the declaring model.
        source: PropertyResolutionError,
    },
    /// Looking up metadata for a child descriptor failed.
    #[error("model metadata lookup failed at path segment {index}: {source}")]
    MetadataLookup {
        /// Zero-based path position.
        index: usize,
        #[source]
        /// Metadata lookup failure for the next model type.
        source: ModelMetadataError,
    },
    /// Reading or mutably projecting a property failed at runtime.
    #[error("property access failed for `{name}` at path segment {index}: {source}")]
    AccessFailure {
        /// Zero-based path position.
        index: usize,
        /// Property name whose adapter failed.
        name: String,
        #[source]
        /// Underlying property access failure.
        source: Box<PropertyAccessError>,
    },
}
