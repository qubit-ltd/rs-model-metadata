// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Read-only concrete and generic model entries.

use qubit_reflect::identity::FragmentIdentity;

use super::internal::model_entry_target::ModelEntryTarget;
#[cfg(feature = "generic")]
use crate::generic::GenericModelMetadata;
use crate::metadata::ModelId;
use crate::metadata::TypeMetadata;

/// An immutable projection of one model and its registration provenance.
///
/// Entries expose the stable model identifier and the metadata and sources
/// associated with that registry view.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::registry::{ModelEntry, ModelRegistry};
///
/// # mod example {
/// use qubit_model_derive::Model;
///
/// #[Model(id = "example.ModelEntryNote")]
/// pub struct Note { pub title: String }
/// # }
/// # fn main() {
/// let registry = ModelRegistry::try_global().expect("valid model registry");
/// for entry in registry.entries() {
///     let entry: &ModelEntry<'_> = entry;
///     assert!(registry.get(entry.model_id().as_str()).is_some());
/// }
/// assert!(registry.get("example.ModelEntryNote").is_some());
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ModelEntry<'reflection> {
    /// Stable ID used to index this entry.
    pub(super) model_id: ModelId,
    /// Concrete or generic metadata carried by this entry.
    target: ModelEntryTarget,
    /// Metadata capability fragment for snapshot projections, or the explicit
    /// caller-provided source for static entries.
    pub(super) source: &'reflection FragmentIdentity,
    /// Reflected type or definition member that declared this entry.
    declaration_source: Option<&'reflection FragmentIdentity>,
}

impl<'reflection> ModelEntry<'reflection> {
    /// Creates an entry from concrete metadata and borrowed provenance.
    ///
    /// Returns `None` for an anonymous model; `Some` retains its declared ID
    /// without initializing reflection or copying either borrowed input.
    ///
    /// # Parameters
    ///
    /// - `metadata`: static metadata containing the model's stable ID.
    /// - `source`: registration fragment retained by this entry.
    /// - `declaration_source`: optional fragment that declared the metadata
    ///   target.
    ///
    /// # Returns
    ///
    /// `Some` when `metadata` has a model ID; otherwise `None`.
    #[must_use]
    #[inline]
    pub(super) fn concrete(
        metadata: &'static TypeMetadata,
        source: &'reflection FragmentIdentity,
        declaration_source: Option<&'reflection FragmentIdentity>,
    ) -> Option<Self> {
        Some(Self {
            model_id: metadata.model_id()?,
            target: ModelEntryTarget::Concrete(metadata),
            source,
            declaration_source,
        })
    }

    /// Creates an entry from a generic definition and borrowed provenance.
    ///
    /// Returns `None` for an anonymous definition; `Some` retains the template
    /// rather than pretending it has a concrete instance type identity.
    ///
    /// # Parameters
    ///
    /// - `metadata`: generic definition metadata containing its stable model
    ///   ID.
    /// - `source`: registration fragment retained by this entry.
    /// - `declaration_source`: optional fragment that declared the definition.
    ///
    /// # Returns
    ///
    /// `Some` when `metadata` has a model ID; otherwise `None`.
    #[must_use]
    #[inline]
    #[cfg(feature = "generic")]
    pub(super) fn generic(
        metadata: &'static GenericModelMetadata,
        source: &'reflection FragmentIdentity,
        declaration_source: Option<&'reflection FragmentIdentity>,
    ) -> Option<Self> {
        Some(Self {
            model_id: metadata.model_id()?,
            target: ModelEntryTarget::Generic(metadata),
            source,
            declaration_source,
        })
    }

    /// Returns the stable model ID shared by concrete and generic entries.
    ///
    /// # Returns
    ///
    /// The immutable ID used to index the model registry entry.
    #[must_use = "inspect the model identity"]
    #[inline]
    pub const fn model_id(&self) -> ModelId {
        self.model_id
    }

    /// Returns the metadata capability fragment for snapshot projections, or
    /// the source supplied to a static registry constructor.
    ///
    /// # Returns
    ///
    /// The registration fragment borrowed for this entry's lifetime.
    #[must_use]
    #[inline]
    pub const fn source(&self) -> &'reflection FragmentIdentity {
        self.source
    }

    /// Returns the reflected type or generic definition source for snapshot
    /// projections, or `None` for entries built from static metadata.
    ///
    /// # Returns
    ///
    /// The declaring fragment for a snapshot entry, or `None` for a static
    /// entry.
    #[must_use]
    #[inline]
    pub const fn declaration_source(&self) -> Option<&'reflection FragmentIdentity> {
        self.declaration_source
    }

    /// Returns concrete metadata, or `None` for a generic declaration.
    ///
    /// # Returns
    ///
    /// The concrete type metadata, or `None` when this entry is generic.
    #[must_use]
    #[inline]
    pub const fn metadata(self) -> Option<&'static TypeMetadata> {
        match self.target {
            ModelEntryTarget::Concrete(metadata) => Some(metadata),
            #[cfg(feature = "generic")]
            ModelEntryTarget::Generic(_) => None,
        }
    }

    /// Returns generic metadata, or `None` for a concrete entry.
    ///
    /// # Returns
    ///
    /// The generic definition metadata, or `None` when this entry is concrete.
    #[must_use]
    #[inline]
    #[cfg(feature = "generic")]
    pub const fn generic_metadata(self) -> Option<&'static GenericModelMetadata> {
        match self.target {
            ModelEntryTarget::Concrete(_) => None,
            ModelEntryTarget::Generic(metadata) => Some(metadata),
        }
    }
}
