// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Frozen indexes projected from concrete and generic reflection capabilities.

#![allow(clippy::result_large_err)]

#[path = "path_cache.rs"]
mod path_cache;

use std::any::TypeId;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;

#[cfg(feature = "generic")]
use qubit_reflect::TypeDefinitionId;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::capability::CapabilityLookup;
use qubit_reflect::identity::FragmentIdentity;
use qubit_reflect::registry::CapabilityTarget;
use qubit_reflect::registry::ReflectRegistry;

use self::path_cache::PathCache;
use self::path_cache::PathCacheKey;
use super::ModelRegistryError;
use super::model_entry::ModelEntry;
use crate::PropertyAccessPath;
use crate::PropertyAccessPathError;
#[cfg(feature = "generic")]
use crate::generic::GenericModelMetadata;
use crate::metadata::ModelId;
use crate::metadata::ModelMetadataError;
use crate::metadata::PropertyResolutionError;
use crate::metadata::ResolvedProperties;
use crate::metadata::TypeMetadata;
#[cfg(feature = "generic")]
use crate::reflect_facade::generic_model_metadata_key;
use crate::reflect_facade::model_metadata_key;

/// Identifies cached properties by exact Rust type and metadata allocation.
///
/// The pointer component distinguishes multiple static metadata values that
/// share one Rust type identity.
type PropertyCacheKey = (TypeId, usize);

/// Shares one lazy property-resolution result among concurrent lookups.
///
/// The cell retains either the resolved properties or their original error.
type PropertyCacheCell = Arc<OnceLock<Result<ResolvedProperties, PropertyResolutionError>>>;

/// Protects the per-registry map of lazily initialized property results.
type PropertyCache = Mutex<HashMap<PropertyCacheKey, PropertyCacheCell>>;

/// Identifies metadata lookups by exact Rust type and descriptor allocation.
type MetadataCacheKey = (TypeId, usize);

/// Retains successful, missing, and failed metadata lookup results.
type MetadataCacheValue = Result<Option<&'static TypeMetadata>, ModelMetadataError>;

/// Shares one lazy metadata lookup among concurrent callers.
type MetadataCacheCell = Arc<OnceLock<MetadataCacheValue>>;

/// Protects the per-registry map of metadata lookup cells.
type MetadataCache = Mutex<HashMap<MetadataCacheKey, MetadataCacheCell>>;

/// An immutable registry sorted by stable model ID and fragment identity.
///
/// The lifetime retains borrowed reflection provenance. Metadata itself lives
/// for the process. Use an explicit registry for isolation;
/// [`Self::try_global`] initializes and caches the linked reflection registry,
/// including failures. Anonymous roots are supplied directly to the structural
/// resolver instead of being indexed under an invented stable ID.
///
/// # Type Parameters
///
/// - `'reflection`: Lifetime of the borrowed reflection provenance retained by
///   this registry.
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
    /// Lookup from process-local definition identity to generic metadata.
    #[cfg(feature = "generic")]
    generic_definition_indices: BTreeMap<TypeDefinitionId, &'static GenericModelMetadata>,
    /// Reflection snapshot that owns effective capability resolution.
    reflection: Option<&'reflection ReflectRegistry>,
    /// Per-registry cache for snapshot-specific property resolutions.
    property_cache: PropertyCache,
    /// Per-registry cache for validated metadata lookups.
    metadata_cache: MetadataCache,
    /// Bounded successful access paths retained only by this registry.
    path_cache: Mutex<PathCache<PropertyAccessPath>>,
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
    /// # Parameters
    ///
    /// - `reflection`: Frozen registry whose declarations and capabilities are
    ///   projected into this model registry.
    ///
    /// # Returns
    ///
    /// The immutable projection, borrowing provenance from `reflection`.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRegistryError`] for capability failures, model metadata
    /// capability targets that are not snapshot members, duplicate model
    /// IDs or generic definition identities, conflicting concrete type
    /// registrations, or metadata inconsistent with its reflected
    /// descriptor/definition. ABI failures retain their cause.
    ///
    /// # Panics
    ///
    /// Propagates a panic from a registered metadata provider.
    #[must_use = "handle invalid model registrations"]
    pub fn from_reflect_registry(reflection: &'reflection ReflectRegistry) -> Result<Self, ModelRegistryError> {
        if let Some((type_id, source)) = reflection
            .capability_only_type_targets(model_metadata_key().id().as_str())
            .first()
        {
            return Err(ModelRegistryError::unregistered_model_target(
                CapabilityTarget::Type(*type_id),
                *model_metadata_key().id(),
                (*source).clone(),
            ));
        }
        #[cfg(feature = "generic")]
        if let Some((definition_id, source)) = reflection
            .capability_only_definition_targets(generic_model_metadata_key().id().as_str())
            .first()
        {
            return Err(ModelRegistryError::unregistered_model_target(
                CapabilityTarget::TypeDefinition(*definition_id),
                *generic_model_metadata_key().id(),
                (*source).clone(),
            ));
        }
        let mut entries = Vec::new();
        let mut metadata_inputs = Vec::new();
        #[cfg(feature = "generic")]
        let mut generic_inputs = Vec::new();
        for member in reflection.type_capability_members(model_metadata_key()) {
            let descriptor = member.target();
            let provider = match member.lookup() {
                CapabilityLookup::Found(provider) => *provider,
                CapabilityLookup::FactOnly(capability) => {
                    return Err(ModelRegistryError::fact_only_capability(
                        *capability.id(),
                        member.origin().clone(),
                    ));
                }
                CapabilityLookup::AdapterTypeMismatch {
                    descriptor: capability,
                    expected,
                } => {
                    return Err(ModelRegistryError::adapter_type_mismatch(
                        *capability.id(),
                        *expected,
                        capability.adapter_type(),
                        member.origin().clone(),
                    ));
                }
                CapabilityLookup::Missing => {
                    unreachable!("capability members always contain a fact")
                }
            };
            let capability_source = member.source();
            let metadata = provider();
            if let Err(cause) = metadata.validate_descriptor(descriptor) {
                return Err(ModelRegistryError::invalid_abi(
                    metadata.model_id(),
                    vec![capability_source.clone()],
                    cause,
                ));
            }
            metadata_inputs.push((metadata, capability_source));
            if metadata.model_id().is_some() {
                entries.push(
                    ModelEntry::concrete(
                        metadata,
                        capability_source,
                        Some(
                            reflection
                                .type_source(descriptor.type_id())
                                .expect("capability member types retain declaration source"),
                        ),
                    )
                    .expect("metadata with a model ID creates an entry"),
                );
            }
        }
        #[cfg(feature = "generic")]
        for member in reflection.definition_capability_members(generic_model_metadata_key()) {
            let definition = member.target();
            let provider = match member.lookup() {
                CapabilityLookup::Found(provider) => *provider,
                CapabilityLookup::FactOnly(capability) => {
                    return Err(ModelRegistryError::fact_only_capability(
                        *capability.id(),
                        member.origin().clone(),
                    ));
                }
                CapabilityLookup::AdapterTypeMismatch {
                    descriptor: capability,
                    expected,
                } => {
                    return Err(ModelRegistryError::adapter_type_mismatch(
                        *capability.id(),
                        *expected,
                        capability.adapter_type(),
                        member.origin().clone(),
                    ));
                }
                CapabilityLookup::Missing => {
                    unreachable!("capability members always contain a fact")
                }
            };
            let capability_source = member.source();
            let metadata = provider();
            if metadata.definition().id() != definition.id() {
                return Err(ModelRegistryError::conflict(
                    metadata.model_id(),
                    vec![capability_source.clone()],
                ));
            }
            generic_inputs.push((
                metadata,
                capability_source,
                Some(
                    reflection
                        .definition_source(definition.id())
                        .expect("capability member definitions retain declaration source"),
                ),
            ));
        }
        let mut registry = Self::build(
            entries,
            #[cfg(feature = "generic")]
            generic_inputs,
            metadata_inputs,
        )?;
        registry.reflection = Some(reflection);
        Ok(registry)
    }

    /// Builds an isolated deterministic registry from explicit metadata.
    ///
    /// Borrows each registration's provenance and retains its static metadata.
    /// No metadata provider or global registry is invoked. Supply anonymous
    /// models as structural roots instead of registrations under stable IDs.
    /// Property resolution uses only the model's statically declared local
    /// properties and does not read independently registered `ModelImpl`
    /// capabilities.
    ///
    /// # Type Parameters
    ///
    /// - `'a`: Lifetime of each borrowed declaration source retained by the
    ///   returned registry.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRegistryError`] for an anonymous concrete registration,
    /// a repeated stable model ID, or conflicting registrations of one concrete
    /// Rust type. Concrete metadata whose ABI does not match its reflected
    /// descriptor is rejected with its ABI cause.
    ///
    /// # Parameters
    ///
    /// - `concrete`: Static metadata and declaration provenance to index.
    ///
    /// # Returns
    ///
    /// The isolated registry borrowing the supplied provenance.
    #[must_use = "handle invalid model registrations"]
    pub fn from_static_metadata<'a>(
        concrete: &[(&'static TypeMetadata, &'a FragmentIdentity)],
    ) -> Result<ModelRegistry<'a>, ModelRegistryError> {
        let mut entries = Vec::with_capacity(concrete.len());
        for &(metadata, source) in concrete {
            if let Err(cause) = metadata.validate_descriptor(metadata.descriptor()) {
                return Err(ModelRegistryError::invalid_abi(
                    metadata.model_id(),
                    vec![source.clone()],
                    cause,
                ));
            }
            let Some(entry) = ModelEntry::concrete(metadata, source, None) else {
                return Err(ModelRegistryError::conflict(None, vec![source.clone()]));
            };
            entries.push(entry);
        }
        ModelRegistry::<'a>::build(
            entries,
            #[cfg(feature = "generic")]
            Vec::new(),
            concrete.to_vec(),
        )
    }

    /// Builds an isolated registry from concrete and generic declarations.
    ///
    /// Retains borrowed provenance without invoking providers or global state.
    /// Anonymous generic definitions remain available by definition identity;
    /// concrete anonymous registrations are rejected. Property resolution uses
    /// only the model's statically declared local properties and does not read
    /// independently registered `ModelImpl` capabilities.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRegistryError`] for an anonymous concrete registration,
    /// duplicate stable IDs or generic definition identities across
    /// declarations, conflicting registrations of one concrete Rust type, or
    /// concrete metadata whose ABI does not match its reflected descriptor.
    /// ABI errors retain their cause.
    ///
    /// # Type Parameters
    ///
    /// - `'a`: Lifetime of the borrowed declaration provenance retained by the
    ///   returned registry.
    ///
    /// # Parameters
    ///
    /// - `concrete`: Static concrete metadata and provenance to index.
    /// - `generic`: Static generic metadata and provenance to index.
    ///
    /// # Returns
    ///
    /// The isolated registry borrowing the supplied provenance.
    #[must_use = "handle invalid model registrations"]
    #[cfg(feature = "generic")]
    pub fn from_static_metadata_with_generics<'a>(
        concrete: &[(&'static TypeMetadata, &'a FragmentIdentity)],
        generic: &[(&'static GenericModelMetadata, &'a FragmentIdentity)],
    ) -> Result<ModelRegistry<'a>, ModelRegistryError> {
        let mut entries = Vec::with_capacity(concrete.len() + generic.len());
        for &(metadata, source) in concrete {
            if let Err(cause) = metadata.validate_descriptor(metadata.descriptor()) {
                return Err(ModelRegistryError::invalid_abi(
                    metadata.model_id(),
                    vec![source.clone()],
                    cause,
                ));
            }
            let Some(entry) = ModelEntry::concrete(metadata, source, None) else {
                return Err(ModelRegistryError::conflict(None, vec![source.clone()]));
            };
            entries.push(entry);
        }
        ModelRegistry::<'a>::build(
            entries,
            generic
                .iter()
                .map(|&(metadata, source)| (metadata, source, None))
                .collect(),
            concrete.to_vec(),
        )
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
    /// # Returns
    ///
    /// A reference to the process-wide registry after successful
    /// initialization.
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
    ///
    /// # Returns
    ///
    /// The process-wide validated registry.
    #[must_use]
    pub fn global() -> &'static ModelRegistry<'static> {
        Self::try_global().unwrap_or_else(|error| panic!("invalid global model registry: {error}"))
    }

    /// Validates and indexes owned registrations in deterministic order.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRegistryError`] for duplicate model IDs or generic
    /// definition identities, or inconsistent concrete registration metadata.
    ///
    /// # Parameters
    ///
    /// - `entries`: Concrete and generic model entries to validate and index.
    /// - `generic_inputs`: Generic definitions and their declaration provenance
    ///   when generic metadata is enabled.
    /// - `metadata_inputs`: Concrete metadata and its source validated before
    ///   registry build, including anonymous metadata without a stable ID.
    ///
    /// # Returns
    ///
    /// The registry with owned indexes, an empty property cache, and no
    /// reflection snapshot associated yet.
    fn build(
        mut entries: Vec<ModelEntry<'reflection>>,
        #[cfg(feature = "generic")] mut generic_inputs: Vec<(
            &'static GenericModelMetadata,
            &'reflection FragmentIdentity,
            Option<&'reflection FragmentIdentity>,
        )>,
        metadata_inputs: Vec<(&'static TypeMetadata, &'reflection FragmentIdentity)>,
    ) -> Result<Self, ModelRegistryError> {
        #[cfg(feature = "generic")]
        {
            generic_inputs.sort_by(|(left, left_source, _), (right, right_source, _)| {
                left_source
                    .cmp(right_source)
                    .then_with(|| left.definition().rust_path().cmp(right.definition().rust_path()))
            });
        }
        #[cfg(feature = "generic")]
        let mut generic_definition_indices = BTreeMap::new();
        #[cfg(feature = "generic")]
        let mut definition_sources = BTreeMap::new();
        #[cfg(feature = "generic")]
        let mut generic_definitions = Vec::with_capacity(generic_inputs.len());
        #[cfg(feature = "generic")]
        for &(metadata, source, _) in &generic_inputs {
            let definition_id = metadata.definition().id();
            if let Some(previous_source) = definition_sources.insert(definition_id, source) {
                return Err(ModelRegistryError::conflict(
                    metadata.model_id(),
                    vec![previous_source.clone(), source.clone()],
                ));
            }
            generic_definition_indices.insert(definition_id, metadata);
            generic_definitions.push(metadata);
        }
        #[cfg(feature = "generic")]
        entries.extend(
            generic_inputs
                .iter()
                .filter_map(|&(metadata, source, declaration_source)| {
                    ModelEntry::generic(metadata, source, declaration_source)
                }),
        );
        entries.sort_by(compare_entries);
        for pair in entries.windows(2) {
            if pair[0].model_id == pair[1].model_id {
                let sources = pair.iter().map(|entry| entry.source.clone()).collect();
                return Err(ModelRegistryError::duplicate(pair[0].model_id, sources));
            }
        }

        let mut indices = BTreeMap::new();
        let mut type_indices = HashMap::new();
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
        }

        let mut metadata_cache_with_sources: HashMap<
            MetadataCacheKey,
            (MetadataCacheCell, &'reflection FragmentIdentity),
        > = HashMap::with_capacity(metadata_inputs.len());
        for (metadata, source) in metadata_inputs {
            let descriptor = metadata.descriptor();
            let key = (descriptor.type_id(), descriptor as *const TypeDescriptor as usize);
            if let Some((cell, first_source)) = metadata_cache_with_sources.get(&key) {
                let cached = cell.get().expect("prefilled metadata cell").as_ref();
                if !matches!(cached, Ok(Some(existing)) if std::ptr::eq(*existing, metadata)) {
                    return Err(ModelRegistryError::conflict(
                        metadata.model_id(),
                        vec![(*first_source).clone(), source.clone()],
                    ));
                }
                continue;
            }
            let cell = Arc::new(OnceLock::new());
            cell.set(Ok(Some(metadata))).expect("new metadata cache cell");
            metadata_cache_with_sources.insert(key, (cell, source));
        }
        let metadata_cache: HashMap<MetadataCacheKey, MetadataCacheCell> =
            metadata_cache_with_sources
                .into_iter()
                .map(|(key, (cell, _source))| (key, cell))
                .collect();

        Ok(Self {
            entries: entries.into_boxed_slice(),
            indices,
            type_indices,
            #[cfg(feature = "generic")]
            generic_definitions: generic_definitions.into_boxed_slice(),
            #[cfg(feature = "generic")]
            generic_definition_indices,
            reflection: None,
            property_cache: Mutex::default(),
            metadata_cache: Mutex::new(metadata_cache),
            path_cache: Mutex::default(),
        })
    }

    /// Finds one immutable model entry by stable ID.
    /// Returns `None` for invalid IDs and IDs absent from this registry.
    ///
    /// # Parameters
    ///
    /// - `id`: Stable model ID to look up.
    ///
    /// # Returns
    ///
    /// The indexed entry, or `None` when the ID is invalid or absent.
    #[must_use]
    #[inline]
    pub fn get(&self, id: &str) -> Option<&ModelEntry<'reflection>> {
        if ModelId::validate(id).is_err() {
            return None;
        }
        self.entries.get(*self.indices.get(id)?)
    }

    /// Enumerates concrete and generic models in stable model-ID order.
    ///
    /// # Returns
    ///
    /// All entries in deterministic model-ID and fragment-identity order.
    #[must_use]
    #[inline]
    pub fn entries(&self) -> &[ModelEntry<'reflection>] {
        &self.entries
    }

    /// Returns concrete metadata for a stable ID, or `None` for an invalid,
    /// absent, or generic-definition-only ID.
    ///
    /// # Parameters
    ///
    /// - `id`: Stable model ID to resolve.
    ///
    /// # Returns
    ///
    /// Concrete metadata for a registered concrete model; `None` for an
    /// invalid ID, an absent ID, or a generic-definition-only registration.
    #[must_use]
    #[inline]
    pub fn metadata(&self, id: &str) -> Option<&'static TypeMetadata> {
        self.get(id).and_then(|entry| entry.metadata())
    }

    /// Returns generic-definition metadata for a stable ID, or `None` for an
    /// invalid, absent, or concrete-only ID.
    ///
    /// # Parameters
    ///
    /// - `id`: Stable model ID to resolve.
    ///
    /// # Returns
    ///
    /// Generic metadata for a registered definition; `None` for an invalid ID,
    /// an absent ID, or a concrete-only registration.
    #[must_use]
    #[inline]
    #[cfg(feature = "generic")]
    pub fn generic(&self, id: &str) -> Option<&'static GenericModelMetadata> {
        self.get(id).and_then(|entry| entry.generic_metadata())
    }

    /// Returns registered concrete metadata by exact Rust identity, or `None`
    /// when that concrete type is absent from this registry.
    ///
    /// # Parameters
    ///
    /// - `type_id`: Exact Rust type identity to look up.
    ///
    /// # Returns
    ///
    /// Registered concrete metadata, or `None` when that type is absent.
    #[must_use]
    #[inline]
    pub fn by_type_id(&self, type_id: TypeId) -> Option<&'static TypeMetadata> {
        self.entries
            .get(*self.type_indices.get(&type_id)?)
            .and_then(|entry| entry.metadata())
    }

    /// Returns model metadata resolved for one exact concrete descriptor.
    ///
    /// Reuses the validated result for this exact descriptor within this
    /// registry. Snapshot projections prefill metadata supplied during
    /// construction, including anonymous metadata. Other lookups cache their
    /// provider result, absence, or structured error after the first call. A
    /// provider panic leaves the cache cell uninitialized and can be retried.
    /// No global registry is consulted.
    ///
    /// # Parameters
    ///
    /// - `descriptor`: Exact static type descriptor whose metadata is queried.
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
        let key = (descriptor.type_id(), descriptor as *const TypeDescriptor as usize);
        let cell = {
            let mut cache = self.metadata_cache.lock().expect("metadata cache lock");
            Arc::clone(cache.entry(key).or_insert_with(|| Arc::new(OnceLock::new())))
        };
        cell.get_or_init(|| {
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
        })
        .clone()
    }

    /// Resolves properties using this model registry's reflection snapshot.
    ///
    /// Registries built by `from_static_metadata` or
    /// `from_static_metadata_with_generics` use only
    /// the model's statically declared local properties, without independently
    /// registered `ModelImpl` capabilities. Snapshot registries invoke their
    /// implementation providers on a cache miss, then reuse the owned merge
    /// for the same metadata identity. Cache storage is released with this
    /// registry. This never consults global state.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyResolutionError`] for capability or property assembly
    /// failures, retaining the original diagnostics.
    ///
    /// # Parameters
    ///
    /// - `metadata`: Static model metadata whose local and reflected properties
    ///   are resolved.
    ///
    /// # Returns
    ///
    /// The merged or static resolved properties on success.
    ///
    /// # Panics
    ///
    /// Propagates a panic from an implementation provider or a poisoned cache
    /// lock.
    #[must_use = "handle property resolution failures"]
    pub fn properties_for(
        &self,
        metadata: &'static TypeMetadata,
    ) -> Result<ResolvedProperties, PropertyResolutionError> {
        let Some(reflection) = self.reflection else {
            return Ok(ResolvedProperties::Static(metadata.local_properties()));
        };
        let key = (metadata.type_id(), metadata as *const TypeMetadata as usize);
        let cell = {
            let mut cache = self.property_cache.lock().expect("property cache lock");
            Arc::clone(cache.entry(key).or_insert_with(|| Arc::new(OnceLock::new())))
        };
        cell.get_or_init(|| metadata.try_properties_in(reflection)).clone()
    }

    /// Compiles or reuses a successful readable path within this registry.
    ///
    /// The key includes the exact root metadata address and all segments.
    /// Failed compilations are retried on later calls. Each registry retains
    /// at most 256 successful paths and evicts the oldest inserted entry when
    /// full, so pointer identity is shared only while an entry remains cached.
    /// A miss compiles outside the cache lock, so concurrent misses may compile
    /// more than once.
    ///
    /// # Errors
    ///
    /// Returns the original [`PropertyAccessPathError`] for invalid segments,
    /// unresolved metadata, and unreadable or unsupported properties.
    ///
    /// # Panics
    ///
    /// Panics if the cache mutex was poisoned.
    #[must_use = "handle access path compilation failures"]
    pub fn compile_read_path_cached(
        &self,
        root: &'static TypeMetadata,
        segments: &[&str],
    ) -> Result<Arc<PropertyAccessPath>, PropertyAccessPathError> {
        self.compile_path_cached(root, segments, false)
    }

    /// Compiles or reuses a successful writable path within this registry.
    ///
    /// Write paths have separate entries from read paths. The leaf's runtime
    /// writability remains checked by the compiled path when writing. Each
    /// registry retains at most 256 successful paths, so pointer identity is
    /// shared only while an entry remains cached.
    ///
    /// # Errors
    ///
    /// Returns the original [`PropertyAccessPathError`] for invalid segments,
    /// unresolved metadata, and unreadable or unsupported intermediates.
    ///
    /// # Panics
    ///
    /// Panics if the cache mutex was poisoned.
    #[must_use = "handle access path compilation failures"]
    pub fn compile_write_path_cached(
        &self,
        root: &'static TypeMetadata,
        segments: &[&str],
    ) -> Result<Arc<PropertyAccessPath>, PropertyAccessPathError> {
        self.compile_path_cached(root, segments, true)
    }

    /// Looks up the keyed mode, compiling without a cache lock on a miss.
    ///
    /// Returns the retained `Arc` on success and preserves compilation errors.
    /// A second lookup under the insertion lock resolves concurrent misses.
    fn compile_path_cached(
        &self,
        root: &'static TypeMetadata,
        segments: &[&str],
        write: bool,
    ) -> Result<Arc<PropertyAccessPath>, PropertyAccessPathError> {
        let type_id = root.type_id();
        let metadata_address = root as *const _ as usize;
        {
            let mut cache = self.path_cache.lock().expect("path cache lock");
            if let Some(hit) = cache.get_borrowed(type_id, metadata_address, segments, write) {
                return Ok(hit);
            }
        }
        let compiled = Arc::new(if write {
            PropertyAccessPath::compile_for_write(self, root, segments)?
        } else {
            PropertyAccessPath::compile(self, root, segments)?
        });
        let mut cache = self.path_cache.lock().expect("path cache lock");
        if let Some(hit) = cache.get_borrowed(type_id, metadata_address, segments, write) {
            return Ok(hit);
        }
        let key = PathCacheKey::new(type_id, metadata_address, segments, write);
        Ok(cache.insert_or_existing(key, compiled))
    }

    /// Returns registered generic metadata for one definition identity, or
    /// `None` when no indexed definition matches. This does not instantiate a
    /// concrete model or consult the global registry.
    ///
    /// # Parameters
    ///
    /// - `definition_id`: Process-local identity of the generic definition.
    ///
    /// # Returns
    ///
    /// The indexed generic metadata, or `None` when no definition matches.
    #[must_use]
    #[inline]
    #[cfg(feature = "generic")]
    pub fn generic_metadata_for(&self, definition_id: TypeDefinitionId) -> Option<&'static GenericModelMetadata> {
        self.generic_definition_indices.get(&definition_id).copied()
    }

    /// Returns the metadata capability source for snapshot projections or the
    /// explicit input source for static metadata, or `None` for an invalid or
    /// absent ID.
    ///
    /// # Parameters
    ///
    /// - `id`: Stable model ID whose registration source is requested.
    ///
    /// # Returns
    ///
    /// The borrowed capability or explicit-input source, or `None` when the ID
    /// is invalid or absent.
    #[must_use]
    #[inline]
    pub fn source(&self, id: &str) -> Option<&'reflection FragmentIdentity> {
        Some(self.get(id)?.source)
    }

    /// Returns registered generic definitions ordered by fragment identity and
    /// then Rust path, including definitions without stable model IDs.
    ///
    /// # Returns
    ///
    /// All registered generic definitions in deterministic order.
    #[must_use]
    #[cfg(feature = "generic")]
    #[inline]
    pub fn generic_definitions(&self) -> &[&'static GenericModelMetadata] {
        &self.generic_definitions
    }

    /// Iterates over concrete registrations in stable registry order.
    ///
    /// # Returns
    ///
    /// An iterator over each concrete metadata value and its borrowed source.
    #[must_use = "consume the concrete registration iterator"]
    #[inline]
    pub(crate) fn concrete_entries(
        &self,
    ) -> impl Iterator<Item = (&'static TypeMetadata, &'reflection FragmentIdentity)> + '_ {
        self.entries
            .iter()
            .filter_map(|entry| entry.metadata().map(|metadata| (metadata, entry.source)))
    }
}

/// Compares registrations by stable model ID and then fragment identity.
///
/// # Parameters
///
/// - `left`: First registration to compare.
/// - `right`: Second registration to compare.
///
/// # Returns
///
/// Their deterministic order: model ID first, then fragment identity.
#[inline]
fn compare_entries(left: &ModelEntry, right: &ModelEntry) -> Ordering {
    left.model_id
        .cmp(&right.model_id)
        .then_with(|| left.source.cmp(right.source))
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use qubit_reflect::Reflect;
    use qubit_reflect::TypeDescriptor;
    use qubit_reflect::identity::FragmentIdentity;

    use super::ModelRegistry;
    use crate::__private::v7;
    use crate::metadata::TypeMetadata;
    use crate::registry::ModelRegistryErrorKind;

    #[derive(Reflect)]
    #[reflect(crate = crate)]
    struct AnonymousMetadataFixture;

    /// Creates distinct ABI-valid anonymous metadata values for one descriptor.
    fn anonymous_metadata() -> &'static TypeMetadata {
        v7::leak(
            v7::GeneratedTypeMetadataBuilder::new(
                TypeDescriptor::of::<AnonymousMetadataFixture>(),
                None,
                &[],
                v7::leak(v7::model_role()),
            )
            .finish::<AnonymousMetadataFixture>(),
        )
    }

    #[test]
    fn test_conflicting_anonymous_metadata_retains_both_sources() {
        let first = anonymous_metadata();
        let second = anonymous_metadata();
        assert!(!std::ptr::eq(first, second));
        assert!(first.validate_descriptor(first.descriptor()).is_ok());
        assert!(second.validate_descriptor(second.descriptor()).is_ok());
        let first_source = FragmentIdentity::new("fixture", "tests", 1, 1, "model", 1);
        let second_source = FragmentIdentity::new("fixture", "tests", 2, 1, "model", 2);

        let error = ModelRegistry::build(
            Vec::new(),
            #[cfg(feature = "generic")]
            Vec::new(),
            vec![(first, &first_source), (second, &second_source)],
        )
        .expect_err("different metadata values for one descriptor must conflict");
        assert_eq!(error.kind(), ModelRegistryErrorKind::RegistrationConflict);
        assert_eq!(error.model_id(), None);
        assert_eq!(error.sources(), &[first_source, second_source]);
    }

    #[test]
    fn test_repeated_anonymous_metadata_pointer_does_not_conflict() {
        let metadata = anonymous_metadata();
        let first_source = FragmentIdentity::new("fixture", "tests", 3, 1, "model", 3);
        let second_source = FragmentIdentity::new("fixture", "tests", 4, 1, "model", 4);

        ModelRegistry::build(
            Vec::new(),
            #[cfg(feature = "generic")]
            Vec::new(),
            vec![(metadata, &first_source), (metadata, &second_source)],
        )
        .expect("repeating one metadata pointer is valid during cache prefill");
    }

    #[test]
    fn test_empty_metadata_registry_exposes_empty_indexes() {
        let registry = ModelRegistry::build(
            Vec::new(),
            #[cfg(feature = "generic")]
            Vec::new(),
            Vec::new(),
        )
        .expect("empty registry is valid");

        let entries = <ModelRegistry<'static>>::entries;
        assert!(entries(&registry).is_empty());
        assert!(registry.get("missing.Model").is_none());
        assert!(registry.source("missing.Model").is_none());
        assert!(registry.by_type_id(TypeId::of::<String>()).is_none());
        assert!(
            registry
                .metadata_for(TypeDescriptor::of::<String>())
                .expect("metadata-only lookup is infallible")
                .is_none()
        );
        #[cfg(feature = "generic")]
        {
            let generic_definitions = <ModelRegistry<'static>>::generic_definitions;
            assert!(generic_definitions(&registry).is_empty());
        }
    }
}
