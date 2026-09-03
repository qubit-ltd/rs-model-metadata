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
use qubit_reflect::capability::CapabilityDescriptor;
use qubit_reflect::capability::CapabilityKey;
use qubit_reflect::identity::CapabilityId;

use crate::ModelImplMetadata;
use crate::TypeMetadata;

/// The typed capability adapter supplied by generated model declarations.
#[doc(hidden)]
pub type ModelMetadataProvider = fn() -> &'static TypeMetadata;

/// The typed capability adapter supplied by `ModelImpl`.
#[doc(hidden)]
pub type ModelImplProvider = fn() -> &'static ModelImplMetadata;

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

/// Extends a reflection root with model metadata lookup.
pub trait ModelDescriptorExt {
    /// Returns the metadata provider attached to this exact descriptor root.
    #[must_use]
    fn model_metadata(&self) -> Option<&'static TypeMetadata>;

    /// Returns whether this exact descriptor root has model metadata.
    #[must_use]
    fn is_model_type(&self) -> bool {
        self.model_metadata().is_some()
    }
}

impl ModelDescriptorExt for TypeDescriptor {
    fn model_metadata(&self) -> Option<&'static TypeMetadata> {
        self.get_capability(model_metadata_key()).map(|provider| {
            let metadata = provider();
            metadata.assert_valid_descriptor(self);
            metadata
        })
    }
}
