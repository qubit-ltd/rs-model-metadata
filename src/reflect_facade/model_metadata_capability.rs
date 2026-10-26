// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Typed capability contract for generated concrete model metadata.

use qubit_reflect::capability::CapabilitySpec;

use super::model_metadata_provider::ModelMetadataProvider;

/// Identifies the metadata adapter attached to a concrete model descriptor.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::__private::ModelMetadataCapability;
/// use qubit_reflect::capability::CapabilitySpec;
///
/// assert_eq!(ModelMetadataCapability::ID, "qubit.model.metadata.v1");
/// ```
pub struct ModelMetadataCapability;

impl CapabilitySpec for ModelMetadataCapability {
    /// Callback type used by generated registrations to retrieve static
    /// metadata for a concrete model.
    type Adapter = ModelMetadataProvider;

    /// Stable key used to locate concrete model metadata capabilities.
    const ID: &'static str = "qubit.model.metadata.v1";
}
