// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Frozen indexes projected from concrete and generic reflection capabilities.

use std::any::TypeId;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::OnceLock;

use qubit_reflect::TypeDefinitionId;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::identity::FragmentIdentity;
use qubit_reflect::registry::ReflectRegistry;

use super::ModelRegistryError;
use super::model_entry::ModelEntry;
use crate::GenericModelMetadata;
use crate::ModelId;
use crate::TypeMetadata;

/// An immutable registry sorted by stable model ID and fragment identity.
#[derive(Debug)]
pub struct ModelRegistry<'reflection> {
    /// Registrations in deterministic model-ID and fragment-identity order.
    entries: Box<[ModelEntry<'reflection>]>,
    /// Lookup from a stable model ID to a registration index.
    indices: BTreeMap<ModelId, usize>,
    /// Lookup from an exact Rust type identity to a registration index.
    type_indices: HashMap<TypeId, usize>,
    /// Generic definitions retained in deterministic registration order.
    generic_definitions: Box<[&'static GenericModelMetadata]>,
    /// Reflection snapshot that owns effective capability resolution.
    reflection: Option<&'reflection ReflectRegistry>,
}

impl<'reflection> ModelRegistry<'reflection> {
    /// Projects concrete and generic model registrations from one frozen
    /// reflection snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRegistryError`] for duplicate model IDs or inconsistent
    /// concrete registration metadata.
    #[must_use = "handle invalid model registrations"]
    pub fn from_reflect_registry(reflection: &'reflection ReflectRegistry) -> Result<Self, ModelRegistryError> {
        let mut entries = Vec::new();
        for (descriptor, source) in reflection.types_with_identity() {
            let Some(provider) = reflection
                .capability(descriptor, crate::reflect_facade::model_metadata_key())
                .map_err(|error| ModelRegistryError::capability(error, source.clone()))?
            else {
                continue;
            };
            let metadata = provider();
            if metadata.validate_descriptor(descriptor).is_err() {
                return Err(ModelRegistryError::conflict(metadata.model_id(), vec![source.clone()]));
            }
            if metadata.model_id().is_some() {
                entries
                    .push(ModelEntry::concrete(metadata, source).expect("metadata with a model ID creates an entry"));
            }
        }
        for definition in reflection.definitions() {
            let Some(provider) =
                reflection.definition_capability(definition.id(), crate::reflect_facade::generic_model_metadata_key())
            else {
                continue;
            };
            let metadata = provider();
            if metadata.definition().id() != definition.id() {
                let source = reflection
                    .definition_source(definition.id())
                    .expect("registered definitions retain source identity");
                return Err(ModelRegistryError::conflict(
                    Some(metadata.model_id()),
                    vec![source.clone()],
                ));
            }
            let source = reflection
                .definition_source(definition.id())
                .expect("registered definitions retain source identity");
            entries.push(ModelEntry::generic(metadata, source));
        }
        let mut registry = Self::build(entries)?;
        registry.reflection = Some(reflection);
        Ok(registry)
    }

    /// Builds an isolated deterministic registry from explicit metadata.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRegistryError`] when registrations repeat a model ID or
    /// a concrete registration conflicts with its metadata.
    #[must_use = "handle invalid model registrations"]
    pub fn from_metadata(
        concrete: &[(&'static TypeMetadata, &'static FragmentIdentity)],
        generic: &[(&'static GenericModelMetadata, &'static FragmentIdentity)],
    ) -> Result<ModelRegistry<'static>, ModelRegistryError> {
        let mut entries = Vec::with_capacity(concrete.len() + generic.len());
        for &(metadata, source) in concrete {
            let Some(entry) = ModelEntry::concrete(metadata, source) else {
                return Err(ModelRegistryError::conflict(None, vec![source.clone()]));
            };
            entries.push(entry);
        }
        entries.extend(
            generic
                .iter()
                .map(|&(metadata, source)| ModelEntry::generic(metadata, source)),
        );
        ModelRegistry::<'static>::build(entries)
    }

    /// Validates and indexes owned registrations in deterministic order.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRegistryError`] for duplicate model IDs or inconsistent
    /// concrete registration metadata.
    fn build(mut entries: Vec<ModelEntry<'reflection>>) -> Result<Self, ModelRegistryError> {
        entries.sort_by(compare_entries);
        for pair in entries.windows(2) {
            if pair[0].model_id == pair[1].model_id {
                let sources = pair.iter().map(|entry| entry.source.clone()).collect();
                return Err(ModelRegistryError::duplicate(pair[0].model_id, sources));
            }
        }

        let mut indices = BTreeMap::new();
        let mut type_indices = HashMap::new();
        let mut generic_definitions = Vec::new();
        for (index, entry) in entries.iter().copied().enumerate() {
            indices.insert(entry.model_id, index);
            if let Some(metadata) = entry.metadata() {
                if metadata.model_id() != Some(entry.model_id) {
                    return Err(ModelRegistryError::conflict(
                        Some(entry.model_id),
                        vec![entry.source.clone()],
                    ));
                }
                if let Some(previous) = type_indices.insert(metadata.type_id(), index) {
                    return Err(ModelRegistryError::conflict(
                        Some(entry.model_id),
                        vec![entries[previous].source.clone(), entry.source.clone()],
                    ));
                }
            }
            if let Some(generic) = entry.generic_metadata() {
                generic_definitions.push(generic);
            }
        }

        Ok(Self {
            entries: entries.into_boxed_slice(),
            indices,
            type_indices,
            generic_definitions: generic_definitions.into_boxed_slice(),
            reflection: None,
        })
    }

    /// Initializes reflection first, then freezes all linked model
    /// registrations.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRegistryError`] when reflection initialization or model
    /// registration validation fails. The result is cached for the process.
    #[must_use = "handle model registry initialization failure"]
    pub fn try_global() -> Result<&'static ModelRegistry<'static>, ModelRegistryError> {
        static REGISTRY: OnceLock<Result<ModelRegistry<'static>, ModelRegistryError>> = OnceLock::new();
        match REGISTRY.get_or_init(|| {
            ModelRegistry::<'static>::from_reflect_registry(
                ReflectRegistry::initialize().map_err(ModelRegistryError::reflection)?,
            )
        }) {
            Ok(registry) => Ok(registry),
            Err(error) => Err(error.clone()),
        }
    }

    /// Returns the process-wide registry or panics with a stable diagnostic.
    ///
    /// # Panics
    ///
    /// Panics when the cached global registry initialization failed.
    #[must_use]
    pub fn global() -> &'static ModelRegistry<'static> {
        Self::try_global().unwrap_or_else(|error| panic!("invalid global model registry: {error}"))
    }

    /// Finds one immutable model entry by stable ID.
    /// Returns `None` for invalid IDs and IDs absent from this registry.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&ModelEntry<'reflection>> {
        if ModelId::validate(id).is_err() {
            return None;
        }
        self.entries.get(*self.indices.get(id)?)
    }

    /// Enumerates concrete and generic models in stable model-ID order.
    #[must_use]
    pub fn entries(&self) -> &[ModelEntry<'reflection>] {
        &self.entries
    }

    /// Returns concrete metadata for a stable ID.
    #[must_use]
    pub fn metadata(&self, id: &str) -> Option<&'static TypeMetadata> {
        self.get(id).and_then(|entry| entry.metadata())
    }

    /// Returns generic-definition metadata for a stable ID.
    #[must_use]
    pub fn generic(&self, id: &str) -> Option<&'static GenericModelMetadata> {
        self.get(id).and_then(|entry| entry.generic_metadata())
    }

    /// Returns registered concrete metadata by exact Rust identity.
    #[must_use]
    pub fn by_type_id(&self, type_id: TypeId) -> Option<&'static TypeMetadata> {
        self.entries
            .get(*self.type_indices.get(&type_id)?)
            .and_then(|entry| entry.metadata())
    }

    /// Returns model metadata resolved for one exact concrete descriptor.
    ///
    /// `Ok(None)` means no provider or explicit metadata is available.
    /// Intrinsic capability conflicts and descriptor mismatches retain
    /// their exact cause.
    pub fn metadata_for(
        &self,
        descriptor: &'static TypeDescriptor,
    ) -> Result<Option<&'static TypeMetadata>, crate::ModelMetadataError> {
        let provided = match self.reflection {
            Some(reflection) => reflection
                .capability(descriptor, crate::reflect_facade::model_metadata_key())
                .map_err(|source| crate::ModelMetadataError::Capability {
                    type_id: descriptor.type_id(),
                    type_name: descriptor.type_name(),
                    source,
                })?
                .map(|provider| provider()),
            None => None,
        };
        let metadata = provided.or_else(|| self.by_type_id(descriptor.type_id()));
        if let Some(metadata) = metadata {
            metadata
                .validate_descriptor(descriptor)
                .map_err(|source| crate::ModelMetadataError::Abi {
                    type_id: descriptor.type_id(),
                    type_name: descriptor.type_name(),
                    source,
                })?;
        }
        Ok(metadata)
    }

    /// Resolves properties using this model registry's reflection snapshot.
    ///
    /// Explicit metadata-only registries use local field properties. Returns
    /// assembly errors for inconsistent overlays; never consults global state.
    pub fn properties_for(
        &self,
        metadata: &'static TypeMetadata,
    ) -> Result<&'static crate::LocalPropertySet, crate::PropertyResolutionError> {
        self.reflection.map_or_else(
            || Ok(metadata.local_properties()),
            |reflection| metadata.try_properties_in(reflection),
        )
    }

    /// Returns model metadata for one generic declaration identity.
    #[must_use]
    pub fn generic_metadata_for(&self, definition_id: TypeDefinitionId) -> Option<&'static GenericModelMetadata> {
        self.generic_definitions
            .iter()
            .copied()
            .find(|metadata| metadata.definition().id() == definition_id)
    }

    /// Returns the reflection fragment source for a stable model ID.
    #[must_use]
    pub fn source(&self, id: &str) -> Option<&'reflection FragmentIdentity> {
        Some(self.get(id)?.source)
    }

    pub(crate) fn concrete_entries(
        &self,
    ) -> impl Iterator<Item = (&'static TypeMetadata, &'reflection FragmentIdentity)> + '_ {
        self.entries
            .iter()
            .filter_map(|entry| entry.metadata().map(|metadata| (metadata, entry.source)))
    }

    /// Returns registered generic definitions in deterministic order.
    #[must_use]
    #[inline(always)]
    pub fn generic_definitions(&self) -> &[&'static GenericModelMetadata] {
        &self.generic_definitions
    }
}

/// Compares registrations by stable model ID and then fragment identity.
fn compare_entries(left: &ModelEntry, right: &ModelEntry) -> std::cmp::Ordering {
    left.model_id
        .cmp(&right.model_id)
        .then_with(|| left.source.cmp(right.source))
}
