// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Frozen indexes projected from concrete and generic reflection capabilities.

#![allow(clippy::result_large_err)]

use std::any::TypeId;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::OnceLock;

#[cfg(feature = "generic")]
use qubit_reflect::TypeDefinitionId;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::capability::CapabilityAccessError;
use qubit_reflect::capability::CapabilityLookup;
use qubit_reflect::identity::FragmentIdentity;
use qubit_reflect::registry::ReflectRegistry;

use super::ModelRegistryError;
use super::model_entry::ModelEntry;
#[cfg(feature = "generic")]
use crate::generic::GenericModelMetadata;
use crate::metadata::LocalPropertySet;
use crate::metadata::ModelId;
use crate::metadata::ModelMetadataError;
use crate::metadata::PropertyResolutionError;
use crate::metadata::TypeMetadata;
#[cfg(feature = "generic")]
use crate::reflect_facade::generic_model_metadata_key;
use crate::reflect_facade::model_metadata_key;

/// An immutable registry sorted by stable model ID and fragment identity.
///
/// The lifetime retains borrowed reflection provenance. Metadata itself lives
/// for the process. Use an explicit registry for isolation;
/// [`Self::try_global`] initializes and caches the linked reflection registry,
/// including failures. Anonymous roots are supplied directly to the structural
/// resolver instead of being indexed under an invented stable ID.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
///
/// #[Model(id = "example.RegistryNote")]
/// struct Note { title: String }
/// # fn main() {
/// let registry = ModelRegistry::try_global().expect("valid linked registrations");
/// let note = registry.metadata("example.RegistryNote").expect("linked model");
/// assert_eq!(note.type_id(), TypeMetadata::of::<Note>().type_id());
/// assert!(registry.get("example.Missing").is_none());
/// # }
/// ```
#[derive(Debug)]
pub struct ModelRegistry<'reflection> {
    /// Registrations in deterministic model-ID and fragment-identity order.
    entries: Box<[ModelEntry<'reflection>]>,
    /// Lookup from a stable model ID to a registration index.
    indices: BTreeMap<ModelId, usize>,
    /// Lookup from an exact Rust type identity to a registration index.
    type_indices: HashMap<TypeId, usize>,
    /// Generic definitions retained in deterministic registration order.
    #[cfg(feature = "generic")]
    generic_definitions: Box<[&'static GenericModelMetadata]>,
    /// Reflection snapshot that owns effective capability resolution.
    reflection: Option<&'reflection ReflectRegistry>,
}

impl<'reflection> ModelRegistry<'reflection> {
    /// Projects concrete and generic model registrations from one frozen
    /// reflection snapshot.
    ///
    /// Invokes the snapshot's metadata providers and retains borrowed
    /// registration provenance. Anonymous declarations are not indexed by
    /// stable ID. The returned registry uses this snapshot for later capability
    /// and property queries; it does not consult the global registry.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRegistryError`] for capability failures, duplicate model
    /// IDs, conflicting concrete type registrations, or metadata inconsistent
    /// with its reflected descriptor/definition. ABI failures retain their
    /// cause.
    ///
    /// # Panics
    ///
    /// Propagates a panic from a registered metadata provider.
    #[must_use = "handle invalid model registrations"]
    pub fn from_reflect_registry(reflection: &'reflection ReflectRegistry) -> Result<Self, ModelRegistryError> {
        let mut entries = Vec::new();
        for (descriptor, source) in reflection.types_with_identity() {
            let provider = match reflection
                .capability_lookup(descriptor, model_metadata_key())
                .map_err(|error| {
                    ModelRegistryError::capability(
                        CapabilityAccessError::IntrinsicConflict(error),
                        source.clone(),
                        descriptor.type_id(),
                    )
                })? {
                CapabilityLookup::Missing => continue,
                CapabilityLookup::Found(provider) => provider,
                CapabilityLookup::FactOnly(capability) => {
                    let origin = reflection
                        .capability_origin(descriptor, capability.id().as_str())
                        .expect("capability lookup already resolved the capability set")
                        .expect("effective capability retains its origin");
                    return Err(ModelRegistryError::fact_only_capability(*capability.id(), origin));
                }
                CapabilityLookup::AdapterTypeMismatch {
                    descriptor: capability,
                    expected,
                } => {
                    let origin = reflection
                        .capability_origin(descriptor, capability.id().as_str())
                        .expect("capability lookup already resolved the capability set")
                        .expect("effective capability retains its origin");
                    return Err(ModelRegistryError::adapter_type_mismatch(
                        *capability.id(),
                        expected,
                        capability.adapter_type(),
                        origin,
                    ));
                }
            };
            let metadata = provider();
            if let Err(cause) = metadata.validate_descriptor(descriptor) {
                return Err(ModelRegistryError::invalid_abi(
                    metadata.model_id(),
                    vec![source.clone()],
                    cause,
                ));
            }
            if metadata.model_id().is_some() {
                entries
                    .push(ModelEntry::concrete(metadata, source).expect("metadata with a model ID creates an entry"));
            }
        }
        #[cfg(feature = "generic")]
        for definition in reflection.definitions() {
            let Some(provider) = reflection
                .definition_capability(definition.id(), generic_model_metadata_key())
                .map_err(|error| ModelRegistryError::capability_access(error, reflection, definition))?
            else {
                continue;
            };
            let metadata = provider();
            if metadata.definition().id() != definition.id() {
                let source = reflection
                    .definition_source(definition.id())
                    .expect("registered definitions retain source identity");
                return Err(ModelRegistryError::conflict(metadata.model_id(), vec![source.clone()]));
            }
            let source = reflection
                .definition_source(definition.id())
                .expect("registered definitions retain source identity");
            if let Some(entry) = ModelEntry::generic(metadata, source) {
                entries.push(entry);
            }
        }
        let mut registry = Self::build(entries)?;
        registry.reflection = Some(reflection);
        Ok(registry)
    }

    /// Builds an isolated deterministic registry from explicit metadata.
    ///
    /// Borrows each registration's provenance and retains its static metadata.
    /// No metadata provider or global registry is invoked. Supply anonymous
    /// models as structural roots instead of registrations under stable IDs.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRegistryError`] for an anonymous concrete registration,
    /// a repeated stable model ID, or conflicting registrations of one concrete
    /// Rust type.
    #[must_use = "handle invalid model registrations"]
    pub fn from_metadata<'a>(
        concrete: &[(&'static TypeMetadata, &'a FragmentIdentity)],
    ) -> Result<ModelRegistry<'a>, ModelRegistryError> {
        let mut entries = Vec::with_capacity(concrete.len());
        for &(metadata, source) in concrete {
            let Some(entry) = ModelEntry::concrete(metadata, source) else {
                return Err(ModelRegistryError::conflict(None, vec![source.clone()]));
            };
            entries.push(entry);
        }
        ModelRegistry::<'a>::build(entries)
    }

    /// Builds an isolated registry from concrete and generic declarations.
    ///
    /// Retains borrowed provenance without invoking providers or global state.
    /// Anonymous generic definitions are skipped because they have no stable
    /// registration ID; concrete anonymous registrations are rejected.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRegistryError`] for an anonymous concrete registration,
    /// duplicate stable IDs across either kind of declaration, or conflicting
    /// registrations of one concrete Rust type.
    #[must_use = "handle invalid model registrations"]
    #[cfg(feature = "generic")]
    pub fn from_metadata_with_generics<'a>(
        concrete: &[(&'static TypeMetadata, &'a FragmentIdentity)],
        generic: &[(&'static GenericModelMetadata, &'a FragmentIdentity)],
    ) -> Result<ModelRegistry<'a>, ModelRegistryError> {
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
                .filter_map(|&(metadata, source)| ModelEntry::generic(metadata, source)),
        );
        ModelRegistry::<'a>::build(entries)
    }

    /// Initializes reflection first, then freezes all linked model
    /// registrations.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRegistryError`] when reflection initialization or model
    /// registration validation fails. Both successful and failed results are
    /// cached for the process; subsequent failures return clones of the error.
    ///
    /// # Panics
    ///
    /// Propagates a panic from an initializing metadata provider. A panic does
    /// not cache a result. Providers must not recursively enter this global
    /// initialization.
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
    /// Panics when global registry initialization returns an error, or when an
    /// initializing metadata provider panics.
    #[must_use]
    pub fn global() -> &'static ModelRegistry<'static> {
        Self::try_global().unwrap_or_else(|error| panic!("invalid global model registry: {error}"))
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
        #[cfg(feature = "generic")]
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
            #[cfg(feature = "generic")]
            if let Some(generic) = entry.generic_metadata() {
                generic_definitions.push(generic);
            }
        }

        Ok(Self {
            entries: entries.into_boxed_slice(),
            indices,
            type_indices,
            #[cfg(feature = "generic")]
            generic_definitions: generic_definitions.into_boxed_slice(),
            reflection: None,
        })
    }

    /// Finds one immutable model entry by stable ID.
    /// Returns `None` for invalid IDs and IDs absent from this registry.
    #[must_use]
    #[inline]
    pub fn get(&self, id: &str) -> Option<&ModelEntry<'reflection>> {
        if ModelId::validate(id).is_err() {
            return None;
        }
        self.entries.get(*self.indices.get(id)?)
    }

    /// Enumerates concrete and generic models in stable model-ID order.
    #[must_use]
    #[inline(always)]
    pub fn entries(&self) -> &[ModelEntry<'reflection>] {
        &self.entries
    }

    /// Returns concrete metadata for a stable ID, or `None` for an invalid,
    /// absent, or generic-definition-only ID.
    #[must_use]
    #[inline(always)]
    pub fn metadata(&self, id: &str) -> Option<&'static TypeMetadata> {
        self.get(id).and_then(|entry| entry.metadata())
    }

    /// Returns generic-definition metadata for a stable ID, or `None` for an
    /// invalid, absent, or concrete-only ID.
    #[must_use]
    #[inline(always)]
    #[cfg(feature = "generic")]
    pub fn generic(&self, id: &str) -> Option<&'static GenericModelMetadata> {
        self.get(id).and_then(|entry| entry.generic_metadata())
    }

    /// Returns registered concrete metadata by exact Rust identity, or `None`
    /// when that concrete type is absent from this registry.
    #[must_use]
    #[inline(always)]
    pub fn by_type_id(&self, type_id: TypeId) -> Option<&'static TypeMetadata> {
        self.entries
            .get(*self.type_indices.get(&type_id)?)
            .and_then(|entry| entry.metadata())
    }

    /// Returns model metadata resolved for one exact concrete descriptor.
    ///
    /// With a reflection snapshot, invokes its effective metadata provider on
    /// each call before falling back to explicit registrations. No global
    /// registry is consulted. Metadata-only registries use only their index.
    ///
    /// # Returns
    ///
    /// `Ok(Some(metadata))` contains descriptor-checked metadata. `Ok(None)`
    /// means neither a provider nor an explicit registration supplied metadata.
    ///
    /// # Errors
    ///
    /// Returns [`ModelMetadataError`] for capability resolution or descriptor
    /// ABI failures, preserving the exact queried type and underlying cause.
    ///
    /// # Panics
    ///
    /// Propagates a panic from the selected metadata provider.
    #[must_use = "handle metadata lookup failures"]
    pub fn metadata_for(
        &self,
        descriptor: &'static TypeDescriptor,
    ) -> Result<Option<&'static TypeMetadata>, ModelMetadataError> {
        let provided = match self.reflection {
            Some(reflection) => reflection
                .capability(descriptor, model_metadata_key())
                .map_err(|source| ModelMetadataError::Capability {
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
                .map_err(|source| ModelMetadataError::Abi {
                    type_id: descriptor.type_id(),
                    type_name: descriptor.type_name(),
                    source,
                })?;
        }
        Ok(metadata)
    }

    /// Resolves properties using this model registry's reflection snapshot.
    ///
    /// Explicit metadata-only registries use local field properties. Snapshot
    /// registries invoke their implementation providers, then reuse the merge
    /// cached for that owner and ordered provider result set. The returned
    /// properties live for the process. This never consults global state.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyResolutionError`] for capability or property assembly
    /// failures, retaining the original diagnostics.
    ///
    /// # Panics
    ///
    /// Propagates a panic from an implementation provider or a poisoned merge
    /// cache lock.
    #[must_use = "handle property resolution failures"]
    pub fn properties_for(
        &self,
        metadata: &'static TypeMetadata,
    ) -> Result<&'static LocalPropertySet, PropertyResolutionError> {
        self.reflection.map_or_else(
            || Ok(metadata.local_properties()),
            |reflection| metadata.try_properties_in(reflection),
        )
    }

    /// Returns registered generic metadata for one definition identity, or
    /// `None` when no indexed definition matches. This does not instantiate a
    /// concrete model or consult the global registry.
    #[must_use]
    #[cfg(feature = "generic")]
    pub fn generic_metadata_for(&self, definition_id: TypeDefinitionId) -> Option<&'static GenericModelMetadata> {
        self.generic_definitions
            .iter()
            .copied()
            .find(|metadata| metadata.definition().id() == definition_id)
    }

    /// Returns borrowed registration provenance for a stable ID, or `None` for
    /// an invalid or absent ID.
    #[must_use]
    #[inline(always)]
    pub fn source(&self, id: &str) -> Option<&'reflection FragmentIdentity> {
        Some(self.get(id)?.source)
    }

    /// Returns registered generic definitions in deterministic order.
    #[must_use]
    #[inline(always)]
    #[cfg(feature = "generic")]
    pub fn generic_definitions(&self) -> &[&'static GenericModelMetadata] {
        &self.generic_definitions
    }

    /// Iterates over concrete registrations in stable registry order.
    #[must_use = "consume the concrete registration iterator"]
    #[inline(always)]
    pub(crate) fn concrete_entries(
        &self,
    ) -> impl Iterator<Item = (&'static TypeMetadata, &'reflection FragmentIdentity)> + '_ {
        self.entries
            .iter()
            .filter_map(|entry| entry.metadata().map(|metadata| (metadata, entry.source)))
    }
}

/// Compares registrations by stable model ID and then fragment identity.
fn compare_entries(left: &ModelEntry, right: &ModelEntry) -> Ordering {
    left.model_id
        .cmp(&right.model_id)
        .then_with(|| left.source.cmp(right.source))
}
