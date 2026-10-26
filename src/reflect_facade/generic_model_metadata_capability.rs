// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Typed capability for generic model definition metadata.

#[cfg(feature = "generic")]
use qubit_reflect::capability::CapabilitySpec;

#[cfg(feature = "generic")]
use super::generic_model_metadata_provider::GenericModelMetadataProvider;

/// Marks the stable capability used by generated generic model registrations
/// to retrieve a declaration's static metadata.
#[doc(hidden)]
#[cfg(feature = "generic")]
pub struct GenericModelMetadataCapability;

#[cfg(feature = "generic")]
impl CapabilitySpec for GenericModelMetadataCapability {
    /// Callback supplied by generated registrations to retrieve static
    /// metadata for the registered generic model declaration.
    type Adapter = GenericModelMetadataProvider;

    /// Stable key used to find the generic model metadata capability.
    const ID: &'static str = "qubit.model.generic_metadata.v1";
}
