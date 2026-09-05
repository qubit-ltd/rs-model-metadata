// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Read-only concrete and generic model entries.

use qubit_reflect::identity::FragmentIdentity;

use crate::GenericModelMetadata;
use crate::ModelId;
use crate::TypeMetadata;

#[derive(Clone, Copy, Debug)]
enum ModelEntryTarget {
    Concrete(&'static TypeMetadata),
    Generic(&'static GenericModelMetadata),
}

/// An immutable projection of one model and its registration provenance.
#[derive(Clone, Copy, Debug)]
pub struct ModelEntry {
    pub(super) model_id: ModelId,
    target: ModelEntryTarget,
    pub(super) source: &'static FragmentIdentity,
}

impl ModelEntry {
    /// Returns the stable model ID shared by concrete and generic entries.
    #[must_use = "inspect the model identity"]
    pub const fn model_id(&self) -> ModelId {
        self.model_id
    }

    /// Returns the static fragment that contributed this entry.
    #[must_use]
    pub const fn source(&self) -> &'static FragmentIdentity {
        self.source
    }

    /// Creates an entry only when concrete metadata declares a model ID.
    pub(super) fn concrete(metadata: &'static TypeMetadata, source: &'static FragmentIdentity) -> Option<Self> {
        Some(Self {
            model_id: metadata.model_id()?,
            target: ModelEntryTarget::Concrete(metadata),
            source,
        })
    }

    /// Creates an entry for a registered generic declaration.
    pub(super) const fn generic(metadata: &'static GenericModelMetadata, source: &'static FragmentIdentity) -> Self {
        Self {
            model_id: metadata.model_id(),
            target: ModelEntryTarget::Generic(metadata),
            source,
        }
    }

    /// Returns concrete metadata, or `None` for a generic declaration.
    #[must_use]
    pub const fn metadata(self) -> Option<&'static TypeMetadata> {
        match self.target {
            ModelEntryTarget::Concrete(metadata) => Some(metadata),
            ModelEntryTarget::Generic(_) => None,
        }
    }

    /// Returns generic metadata, or `None` for a concrete entry.
    #[must_use]
    pub const fn generic_metadata(self) -> Option<&'static GenericModelMetadata> {
        match self.target {
            ModelEntryTarget::Concrete(_) => None,
            ModelEntryTarget::Generic(metadata) => Some(metadata),
        }
    }
}
