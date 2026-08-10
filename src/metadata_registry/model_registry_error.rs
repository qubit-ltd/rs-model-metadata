// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Construction errors for immutable model registries.

use thiserror::Error;

use crate::model_id::ModelId;
use crate::model_registration::ModelRegistration;

/// A configuration error encountered while constructing a model registry.
#[derive(Debug, Error)]
pub enum ModelRegistryError {
    /// A registration's ID differs from the ID stored in its metadata.
    #[error(
        "model registration ID {registration_id:?} does not match metadata ID {metadata_id:?} for {registration}"
    )]
    MetadataIdMismatch {
        /// The registration with the inconsistent identifier.
        registration: &'static ModelRegistration,
        /// The identifier declared by the registration.
        registration_id: ModelId,
        /// The identifier stored in the registration's metadata.
        metadata_id: ModelId,
    },
    /// Two registrations declare the same stable model identifier.
    #[error("duplicate model ID {id:?}: first {first}; second {second}")]
    DuplicateId {
        /// The duplicated stable model identifier.
        id: ModelId,
        /// The deterministically first conflicting registration.
        first: &'static ModelRegistration,
        /// The deterministically second conflicting registration.
        second: &'static ModelRegistration,
    },
}
