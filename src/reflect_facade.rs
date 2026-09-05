// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Model-specific access to the shared reflection descriptor root.
// qubit-style: allow type-file-name

use qubit_reflect::TypeDescriptor;
use qubit_reflect::capability::CapabilityConflict;
use qubit_reflect::capability::CapabilityDescriptor;
use qubit_reflect::capability::CapabilityKey;
use qubit_reflect::identity::CapabilityId;
use qubit_reflect::registry::ReflectRegistry;

use crate::GenericModelMetadata;
use crate::ModelImplMetadata;
use crate::TypeMetadata;

/// The typed capability adapter supplied by generated model declarations.
#[doc(hidden)]
pub type ModelMetadataProvider = fn() -> &'static TypeMetadata;

/// The typed capability adapter supplied by `ModelImpl`.
#[doc(hidden)]
pub type ModelImplProvider = fn() -> &'static ModelImplMetadata;

/// The typed capability adapter supplied by generic model declarations.
#[doc(hidden)]
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
pub fn generic_model_metadata_key() -> CapabilityKey<GenericModelMetadataProvider> {
    let id = CapabilityId::new("qubit.model.generic_metadata.v1")
        .expect("the generic model metadata capability ID must be valid");
    CapabilityKey::new(id)
}

/// Builds a capability for one generic model declaration.
#[doc(hidden)]
#[must_use]
pub fn generic_model_capability(provider: GenericModelMetadataProvider) -> CapabilityDescriptor {
    CapabilityDescriptor::with_adapter(generic_model_metadata_key(), provider)
}

/// Builds an inline model capability for one concrete reflection monomorph.
#[doc(hidden)]
#[must_use]
pub fn model_capability<T: crate::HasTypeMetadata>() -> CapabilityDescriptor {
    /// Returns the metadata supplied by `T` after descriptor validation.
    fn provide<T: crate::HasTypeMetadata>() -> &'static TypeMetadata {
        let metadata = <T as crate::__private::TypeMetadataProvider>::__type_metadata();
        metadata.assert_valid_for::<T>();
        metadata
    }
    CapabilityDescriptor::with_adapter(model_metadata_key(), provide::<T> as ModelMetadataProvider)
}

/// Returns the generated model-implementation overlay attached to an exact
/// descriptor root in the frozen reflection snapshot.
pub(crate) fn model_impl_metadata(
    descriptor: &TypeDescriptor,
    registry: &ReflectRegistry,
) -> Result<Option<&'static ModelImplMetadata>, CapabilityConflict> {
    Ok(registry
        .capability(descriptor, model_impl_key())?
        .map(|provider| provider()))
}
