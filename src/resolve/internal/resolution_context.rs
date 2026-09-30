// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Descriptor-checked metadata facts scoped to one resolution attempt.

use std::any::TypeId;
use std::collections::HashMap;

use qubit_reflect::TypeDescriptor;

use crate::metadata::ModelMetadataError;
use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;

/// Retains explicit and discovered metadata without changing the registry.
pub(in crate::resolve) struct ResolutionContext<'a> {
    /// The caller's registry, always queried before resolution-local facts.
    registry: &'a ModelRegistry<'a>,
    /// Concrete metadata supplied or discovered during this attempt only.
    known: HashMap<TypeId, &'static TypeMetadata>,
}

impl<'a> ResolutionContext<'a> {
    /// Seeds one attempt with registered concrete models and explicit roots.
    ///
    /// `registry` supplies capabilities; `roots` supply additional concrete
    /// facts. The returned context owns no global or permanent cache.
    ///
    /// # Parameters
    ///
    /// - `registry`: Registry queried before resolution-local metadata.
    /// - `roots`: Additional metadata available only during this attempt.
    ///
    /// # Returns
    ///
    /// A context seeded with the registry's concrete models and the explicit
    /// roots, without modifying the registry.
    pub(in crate::resolve) fn new(registry: &'a ModelRegistry<'a>, roots: &[&'static TypeMetadata]) -> Self {
        let mut context = Self {
            registry,
            known: HashMap::new(),
        };
        for (metadata, _) in registry.concrete_entries() {
            context.remember(metadata);
        }
        for &root in roots {
            context.remember(root);
        }
        context
    }

    /// Returns the borrowed registry for property and stable target-ID queries.
    ///
    /// # Returns
    ///
    /// The registry supplied when this resolution context was created.
    #[must_use]
    #[inline]
    pub(in crate::resolve) const fn registry(&self) -> &'a ModelRegistry<'a> {
        self.registry
    }

    /// Resolves one descriptor, preferring registry results over local facts.
    ///
    /// Returns `Some` only for descriptor-checked metadata, or `None` when
    /// neither source knows the model. Capability errors propagate unchanged;
    /// incompatible local facts return the same typed ABI cause as registry
    /// metadata. Selected registry providers may panic.
    ///
    /// # Parameters
    ///
    /// - `descriptor`: Static runtime descriptor whose metadata is requested.
    ///
    /// # Returns
    ///
    /// `Ok(Some(metadata))` when the registry or local facts provide
    /// descriptor-checked metadata, `Ok(None)` when neither source knows the
    /// type, or an error when lookup or compatibility validation fails.
    ///
    /// # Errors
    ///
    /// Returns the registry's capability error unchanged, or a typed ABI
    /// error when local metadata is incompatible with the descriptor.
    ///
    /// # Panics
    ///
    /// May panic if the selected registry metadata provider panics.
    #[must_use = "handle descriptor metadata failures"]
    pub(in crate::resolve) fn metadata_for(
        &self,
        descriptor: &'static TypeDescriptor,
    ) -> Result<Option<&'static TypeMetadata>, ModelMetadataError> {
        if let Some(metadata) = self.registry.metadata_for(descriptor)? {
            return Ok(Some(metadata));
        }
        let metadata = self.known.get(&descriptor.type_id()).copied();
        if let Some(metadata) = metadata {
            metadata
                .validate_descriptor(descriptor)
                .map_err(|source| ModelMetadataError::Abi {
                    type_id: descriptor.type_id(),
                    type_name: descriptor.type_name(),
                    source,
                })?;
        }
        Ok(metadata)
    }

    /// Records a supplied or discovered model for this attempt, retaining the
    /// first concrete fact for an already known type.
    ///
    /// # Parameters
    ///
    /// - `metadata`: Static model metadata to retain if its type is not known.
    pub(in crate::resolve) fn remember(&mut self, metadata: &'static TypeMetadata) {
        self.known.entry(metadata.type_id()).or_insert(metadata);
    }
}
