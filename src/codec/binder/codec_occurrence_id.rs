// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::any::TypeId;
use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result as FmtResult;

use crate::metadata::CodecSource;
use crate::metadata::ModelIdBuf;
use crate::metadata::TypeMetadata;

/// Process-local identity of one codec binding occurrence.
///
/// This identity is intended for binding lookup within the current process.
/// It includes the owner's [`TypeId`], so equality, hashing, and ordering only
/// identify the same occurrence within that process. A stable model ID
/// returned by [`model`](Self::model) does not make the complete identity
/// stable across processes.
///
/// Do not persist this value, its [`TypeId`], a hash of it, or its `Debug` or
/// `Display` output as an external identity. Consumers that need to identify a
/// declaration across processes must define a stable protocol separately and
/// account for anonymous models.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::codec::CodecOccurrenceId;
/// use qubit_model_metadata::metadata::CodecSource;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// # mod example {
/// use qubit_model_derive::Model;
///
/// #[Model(id = "example.codec.Note")]
/// pub struct Note { pub title: String }
/// # }
///
/// let occurrence = CodecOccurrenceId::new(
///     TypeMetadata::of::<example::Note>(),
///     "title",
///     CodecSource::Field,
/// );
/// assert_eq!(occurrence.property(), "title");
/// assert_eq!(occurrence.source(), CodecSource::Field);
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CodecOccurrenceId {
    /// Stable model identifier when the owner declares one.
    model: Option<ModelIdBuf>,
    /// Concrete Rust type identity used to distinguish anonymous models.
    type_id: TypeId,
    /// Rust type name retained for human-readable diagnostics.
    type_name: &'static str,
    /// Normalized property path, empty for a canonical value codec.
    property: Box<str>,
    /// Declaration location that supplied the codec.
    source: CodecSource,
}

impl CodecOccurrenceId {
    /// Creates a process-local occurrence identity for binding lookup.
    ///
    /// # Parameters
    ///
    /// * `model` - Static metadata for the owning model.
    /// * `property` - Normalized property path, or an empty string for a
    ///   canonical codec.
    /// * `source` - Declaration location within the model metadata.
    ///
    /// # Returns
    ///
    /// An identity containing the model type, property path, and declaration
    /// source, for use within the current process.
    #[must_use]
    pub fn new(model: &'static TypeMetadata, property: impl Into<Box<str>>, source: CodecSource) -> Self {
        Self {
            model: model.model_id().map(ModelIdBuf::from),
            type_id: model.type_id(),
            type_name: model.type_name(),
            property: property.into(),
            source,
        }
    }

    /// Returns the stable ID declared by the owning model, borrowed from this
    /// occurrence.
    ///
    /// # Returns
    ///
    /// `Some` with a reference to the model's stable ID when it has one, or
    /// `None` for an anonymous model. The reference is borrowed from `self`.
    #[inline]
    pub const fn model(&self) -> Option<&ModelIdBuf> {
        self.model.as_ref()
    }

    /// Returns the exact owner identity, including anonymous types.
    ///
    /// # Returns
    ///
    /// The concrete Rust type identity used for exact registration matching.
    #[must_use]
    #[inline]
    pub const fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Returns the normalized property path, empty for canonical value codecs.
    ///
    /// # Returns
    ///
    /// The normalized path, or an empty string for a model's canonical value
    /// codec.
    #[must_use]
    #[inline]
    pub const fn property(&self) -> &str {
        &self.property
    }

    /// Returns the declaration source.
    ///
    /// # Returns
    ///
    /// The model field, canonical value, or nested selector that supplied the
    /// codec.
    #[must_use]
    #[inline]
    pub const fn source(&self) -> CodecSource {
        self.source
    }
}

impl Display for CodecOccurrenceId {
    /// Formats the model type and property path for diagnostics.
    ///
    /// Canonical codecs use the `<canonical>` marker because they have no
    /// property path.
    ///
    /// # Parameters
    ///
    /// * `formatter` - The formatting destination supplied by the caller.
    ///
    /// # Returns
    ///
    /// The result of writing the model type and property path.
    ///
    /// # Errors
    ///
    /// Returns the formatter's error if the identity cannot be written.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        if self.property.is_empty() {
            write!(formatter, "{}::<canonical>", self.type_name)
        } else {
            write!(formatter, "{}::{}", self.type_name, self.property)
        }
    }
}
