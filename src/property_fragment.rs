// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One unmerged field, getter, or setter property declaration.

use qubit_reflect::descriptor::TypeRef;

use crate::metadata::PropertyFragmentSource;

/// Preserves one local source fact before property compatibility is checked.
///
/// The type exposes fragments through generated model metadata. Use the
/// accessors to inspect the property name, declared type, and source
/// declaration.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::PropertyFragment;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// #[Model(id = "example.Note")]
/// struct Note {
///     title: String,
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let metadata = TypeMetadata::of::<Note>();
/// let fragments = metadata.property_fragments()?;
/// let title = fragments
///     .fragments()
///     .iter()
///     .find(|fragment: &&PropertyFragment| fragment.name() == "title")
///     .expect("generated title fragment");
/// assert_eq!(title.name(), "title");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct PropertyFragment {
    /// Canonical public property name.
    name: &'static str,
    /// Exact type declared by this fragment.
    type_ref: &'static TypeRef,
    /// Field/getter/setter declaration source.
    source: PropertyFragmentSource,
}

impl PropertyFragment {
    /// Creates one generated property source fact.
    ///
    /// # Parameters
    ///
    /// - `name`: canonical public property name.
    /// - `type_ref`: exact reflected type of the property.
    /// - `source`: declaration category and metadata.
    ///
    /// # Returns
    ///
    /// A fragment retaining the supplied static type and declaration metadata.
    #[must_use]
    #[inline]
    pub(crate) const fn new(name: &'static str, type_ref: &'static TypeRef, source: PropertyFragmentSource) -> Self {
        Self { name, type_ref, source }
    }

    /// Returns the canonical public property name.
    ///
    /// # Returns
    ///
    /// The name shared by the field and its accessors.
    #[must_use]
    #[inline]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the exact type declared by this source fragment.
    ///
    /// # Returns
    ///
    /// The reflected type descriptor borrowed from static metadata.
    #[must_use]
    #[inline]
    pub const fn type_ref(&self) -> &'static TypeRef {
        self.type_ref
    }

    /// Returns the field, getter, or setter that declared this fragment.
    ///
    /// # Returns
    ///
    /// The source category and its corresponding declaration metadata.
    #[must_use]
    #[inline]
    pub const fn source(&self) -> PropertyFragmentSource {
        self.source
    }
}
