// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Model-specific access to the shared reflection descriptor root.
// qubit-style: allow type-file-name

use std::sync::Arc;

pub(crate) use internal::ModelImplResolution;
use qubit_reflect::capability::CapabilityAccessError;
use qubit_reflect::capability::CapabilityDescriptor;
use qubit_reflect::capability::CapabilityKey;
use qubit_reflect::identity::CapabilityId;
use qubit_reflect::registry::ReflectRegistry;

#[cfg(feature = "generic")]
use crate::generic::GenericModelMetadata;
use crate::metadata::ModelImplMetadata;
use crate::metadata::TypeMetadata;

// Owns private resolution state shared with reflected model metadata.
#[path = "reflect_facade/internal/model_impl_resolution.rs"]
mod internal;

const MODEL_IMPL_BASE_ID: &str = "qubit.model.impl.v1";
const MODEL_IMPL_FRAGMENT_PREFIX: &str = "qubit.model.impl.v1.f";

/// Reports whether an ID belongs to the base or a nonempty fragment slot.
///
/// # Parameters
///
/// - `id`: capability ID to classify.
///
/// # Returns
///
/// `true` for the base model implementation ID or a valid fragment ID.
#[must_use]
#[inline]
fn is_model_impl_capability(id: &str) -> bool {
    id == MODEL_IMPL_BASE_ID || is_model_impl_fragment_capability(id)
}

/// Reports whether an ID has the required nonempty fragment suffix.
///
/// # Parameters
///
/// - `id`: capability ID to inspect.
///
/// # Returns
///
/// `true` when the fragment prefix is followed by at least one character.
#[must_use]
#[inline]
fn is_model_impl_fragment_capability(id: &str) -> bool {
    id.strip_prefix(MODEL_IMPL_FRAGMENT_PREFIX)
        .is_some_and(|suffix| !suffix.is_empty())
}

/// The typed capability adapter supplied by generated model declarations.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::__private::ModelMetadataProvider;
///
/// fn provider_slot(provider: ModelMetadataProvider) -> ModelMetadataProvider {
///     provider
/// }
/// ```
#[doc(hidden)]
pub type ModelMetadataProvider = fn() -> &'static TypeMetadata;

/// The typed capability adapter supplied by `ModelImpl`.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::ModelImplMetadata;
/// use qubit_model_metadata::__private::ModelImplProvider;
///
/// fn provider_slot(provider: ModelImplProvider) -> ModelImplProvider {
///     let _: fn() -> &'static ModelImplMetadata = provider;
///     provider
/// }
/// ```
#[doc(hidden)]
pub type ModelImplProvider = fn() -> &'static ModelImplMetadata;

/// The typed capability adapter supplied by generic model declarations.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::generic::GenericModelMetadata;
///
/// fn provider_slot(provider: fn() -> &'static GenericModelMetadata) -> fn() -> &'static GenericModelMetadata {
///     provider
/// }
/// ```
#[doc(hidden)]
#[cfg(feature = "generic")]
pub type GenericModelMetadataProvider = fn() -> &'static GenericModelMetadata;

/// Returns the stable key used to retrieve a model metadata provider.
///
/// # Returns
///
/// The key used by generated concrete model capabilities.
#[doc(hidden)]
#[must_use]
pub fn model_metadata_key() -> CapabilityKey<ModelMetadataProvider> {
    let id = CapabilityId::new("qubit.model.metadata.v1").expect("the model metadata capability ID must be valid");
    CapabilityKey::new(id)
}

/// Returns the stable key used to retrieve generated property metadata.
///
/// # Returns
///
/// The key used by generated `ModelImpl` capabilities.
#[doc(hidden)]
#[must_use]
pub fn model_impl_key() -> CapabilityKey<ModelImplProvider> {
    let id = CapabilityId::new("qubit.model.impl.v1").expect("the model implementation capability ID must be valid");
    CapabilityKey::new(id)
}

/// Returns the stable key used to retrieve generic model metadata.
///
/// # Returns
///
/// The key used by generic model declaration capabilities.
#[doc(hidden)]
#[must_use]
#[cfg(feature = "generic")]
pub fn generic_model_metadata_key() -> CapabilityKey<GenericModelMetadataProvider> {
    let id = CapabilityId::new("qubit.model.generic_metadata.v1")
        .expect("the generic model metadata capability ID must be valid");
    CapabilityKey::new(id)
}

/// Builds a capability for one generic model declaration.
///
/// # Parameters
///
/// - `provider`: function returning the declaration's static generic metadata.
///
/// # Returns
///
/// A capability descriptor associating the generic metadata key with
/// `provider`.
#[doc(hidden)]
#[must_use]
#[cfg(feature = "generic")]
pub fn generic_model_capability(provider: GenericModelMetadataProvider) -> CapabilityDescriptor {
    CapabilityDescriptor::with_adapter(generic_model_metadata_key(), provider)
}

/// Builds an inline model capability for one concrete reflection monomorph.
///
/// # Type Parameters
///
/// - `T`: concrete reflected type implementing `HasTypeMetadata`.
///
/// # Returns
///
/// A capability descriptor whose adapter validates and returns `T`'s metadata.
#[doc(hidden)]
#[must_use]
pub fn model_capability<T: crate::metadata::HasTypeMetadata>() -> CapabilityDescriptor {
    /// Returns the metadata supplied by `T` after descriptor validation.
    ///
    /// # Type Parameters
    ///
    /// - `T`: concrete type whose generated metadata is validated.
    ///
    /// # Returns
    ///
    /// The validated static metadata for `T`.
    fn provide<T: crate::metadata::HasTypeMetadata>() -> &'static TypeMetadata {
        let metadata = <T as crate::__private::TypeMetadataProvider>::__type_metadata();
        metadata.assert_valid_for::<T>();
        metadata
    }
    CapabilityDescriptor::with_adapter(model_metadata_key(), provide::<T> as ModelMetadataProvider)
}

/// Returns the generated model-implementation overlay attached to an exact
/// descriptor root in the frozen reflection snapshot.
///
/// # Parameters
///
/// - `metadata`: validated metadata for the descriptor being resolved.
/// - `registry`: frozen registry containing its model implementation
///   capabilities.
///
/// # Returns
///
/// `None` when no provider is registered, otherwise static metadata for one
/// provider or an owned merge for multiple providers.
///
/// # Errors
///
/// Returns a capability access error when the registry cannot resolve the
/// descriptor's capabilities or their origins.
///
/// # Panics
///
/// Panics if a resolved model implementation capability has no retained
/// registration origin.
pub(crate) fn model_impl_metadata(
    metadata: &TypeMetadata,
    registry: &ReflectRegistry,
) -> Result<Option<ModelImplResolution>, CapabilityAccessError> {
    let descriptor = metadata.descriptor();
    let capabilities = registry
        .capabilities(descriptor)
        .map_err(CapabilityAccessError::IntrinsicConflict)?;
    let descriptors = capabilities.descriptors();
    let base_pos = descriptors.partition_point(|item| item.id().as_str() < MODEL_IMPL_BASE_ID);
    let base = descriptors
        .get(base_pos)
        .filter(|item| item.id().as_str() == MODEL_IMPL_BASE_ID);
    let start = descriptors.partition_point(|item| item.id().as_str() < MODEL_IMPL_FRAGMENT_PREFIX);
    let fragments = descriptors[start..]
        .iter()
        .take_while(|item| item.id().as_str().starts_with(MODEL_IMPL_FRAGMENT_PREFIX));
    let mut providers = Vec::new();
    for capability in base
        .into_iter()
        .chain(fragments)
        .filter(|item| is_model_impl_capability(item.id().as_str()))
    {
        let id = capability.id();
        if let Some(provider) = registry.capability(descriptor, CapabilityKey::<ModelImplProvider>::new(*id))? {
            let origin = registry
                .capability_origin(descriptor, id.as_str())
                .map_err(CapabilityAccessError::IntrinsicConflict)?
                .expect("a resolved model implementation capability retains its origin");
            providers.push((*provider, origin));
        }
    }
    match providers.as_slice() {
        [] => Ok(None),
        [(provider, _)] => Ok(Some(ModelImplResolution::Static(provider()))),
        _ => Ok(Some(ModelImplResolution::Merged(Arc::new(ModelImplMetadata::merge(
            metadata, &providers,
        ))))),
    }
}

/// Returns the checked key for an independently registered impl fragment.
///
/// # Parameters
///
/// - `name`: capability ID with a nonempty generated fragment suffix.
///
/// # Returns
///
/// A typed key for resolving the fragment's model implementation provider.
///
/// # Panics
///
/// Panics if `name` is not a ModelImpl fragment ID with a nonempty suffix.
#[doc(hidden)]
#[must_use]
pub fn model_impl_fragment_key(name: &'static str) -> CapabilityKey<ModelImplProvider> {
    assert!(
        is_model_impl_fragment_capability(name),
        "invalid ModelImpl fragment capability ID: {name}",
    );
    CapabilityKey::new(CapabilityId::new(name).expect("valid generated impl capability ID"))
}
