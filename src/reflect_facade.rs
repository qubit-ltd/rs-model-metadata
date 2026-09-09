// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Model-specific access to the shared reflection descriptor root.
// qubit-style: allow type-file-name

use qubit_reflect::capability::CapabilityAccessError;
use qubit_reflect::capability::CapabilityDescriptor;
use qubit_reflect::capability::CapabilityKey;
use qubit_reflect::identity::CapabilityId;
use qubit_reflect::registry::ReflectRegistry;

#[cfg(feature = "generic")]
use crate::generic::GenericModelMetadata;
use crate::metadata::ModelImplMetadata;
use crate::metadata::TypeMetadata;

/// The typed capability adapter supplied by generated model declarations.
#[doc(hidden)]
pub type ModelMetadataProvider = fn() -> &'static TypeMetadata;

/// The typed capability adapter supplied by `ModelImpl`.
#[doc(hidden)]
pub type ModelImplProvider = fn() -> &'static ModelImplMetadata;

/// The typed capability adapter supplied by generic model declarations.
#[doc(hidden)]
#[cfg(feature = "generic")]
pub type GenericModelMetadataProvider = fn() -> &'static GenericModelMetadata;

/// Returns the stable key used to retrieve a model metadata provider.
#[doc(hidden)]
#[must_use]
pub fn model_metadata_key() -> CapabilityKey<ModelMetadataProvider> {
    let id = CapabilityId::new("qubit.model.metadata.v1").expect("the model metadata capability ID must be valid");
    CapabilityKey::new(id)
}

/// Returns the stable key used to retrieve generated property metadata.
#[doc(hidden)]
#[must_use]
pub fn model_impl_key() -> CapabilityKey<ModelImplProvider> {
    let id = CapabilityId::new("qubit.model.impl.v1").expect("the model implementation capability ID must be valid");
    CapabilityKey::new(id)
}

/// Returns the stable key used to retrieve generic model metadata.
#[doc(hidden)]
#[must_use]
#[cfg(feature = "generic")]
pub fn generic_model_metadata_key() -> CapabilityKey<GenericModelMetadataProvider> {
    let id = CapabilityId::new("qubit.model.generic_metadata.v1")
        .expect("the generic model metadata capability ID must be valid");
    CapabilityKey::new(id)
}

/// Builds a capability for one generic model declaration.
#[doc(hidden)]
#[must_use]
#[cfg(feature = "generic")]
pub fn generic_model_capability(provider: GenericModelMetadataProvider) -> CapabilityDescriptor {
    CapabilityDescriptor::with_adapter(generic_model_metadata_key(), provider)
}

/// Builds an inline model capability for one concrete reflection monomorph.
#[doc(hidden)]
#[must_use]
pub fn model_capability<T: crate::metadata::HasTypeMetadata>() -> CapabilityDescriptor {
    /// Returns the metadata supplied by `T` after descriptor validation.
    fn provide<T: crate::metadata::HasTypeMetadata>() -> &'static TypeMetadata {
        let metadata = <T as crate::__private::TypeMetadataProvider>::__type_metadata();
        metadata.assert_valid_for::<T>();
        metadata
    }
    CapabilityDescriptor::with_adapter(model_metadata_key(), provide::<T> as ModelMetadataProvider)
}

/// Returns the generated model-implementation overlay attached to an exact
/// descriptor root in the frozen reflection snapshot.
pub(crate) fn model_impl_metadata(
    metadata: &TypeMetadata,
    registry: &ReflectRegistry,
) -> Result<Option<&'static ModelImplMetadata>, CapabilityAccessError> {
    let descriptor = metadata.descriptor();
    let capabilities = registry
        .capabilities(descriptor)
        .map_err(CapabilityAccessError::IntrinsicConflict)?;
    let mut providers = Vec::new();
    for capability in capabilities.descriptors() {
        let id = capability.id();
        if (id.as_str() == "qubit.model.impl.v1" || id.as_str().starts_with("qubit.model.impl.v1.f"))
            && let Some(provider) = registry.capability(descriptor, CapabilityKey::<ModelImplProvider>::new(*id))?
        {
            providers.push(*provider);
        }
    }
    match providers.as_slice() {
        [] => Ok(None),
        [provider] => Ok(Some(provider())),
        _ => Ok(Some(ModelImplMetadata::merge(metadata, &providers))),
    }
}

/// Returns the checked key for an independently registered impl fragment.
#[doc(hidden)]
#[must_use]
pub fn model_impl_fragment_key(name: &'static str) -> CapabilityKey<ModelImplProvider> {
    CapabilityKey::new(CapabilityId::new(name).expect("valid generated impl capability ID"))
}
