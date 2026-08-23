// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Construction errors for immutable model registries.

use thiserror::Error;

use crate::model_id::ModelId;
use crate::model_id::ModelIdError;
use crate::model_registration::ModelRegistration;
use crate::type_metadata::TypeIdentity;

/// A configuration error encountered while constructing a model registry.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ModelRegistryError {
    /// A registration contains an ID that violates the stable-ID protocol.
    #[error("model registration ID {id:?} is invalid for {registration}: {source}")]
    InvalidRegistrationId {
        /// The registration containing the invalid ID.
        registration: &'static ModelRegistration,
        /// The invalid identifier stored by the registration.
        id: ModelId,
        /// The validation failure.
        #[source]
        source: ModelIdError,
    },
    /// A registration's metadata contains an invalid stable ID.
    #[error("model metadata ID {id:?} is invalid for {registration}: {source}")]
    InvalidMetadataId {
        /// The registration whose metadata contains the invalid ID.
        registration: &'static ModelRegistration,
        /// The invalid identifier stored by the metadata.
        id: ModelId,
        /// The validation failure.
        #[source]
        source: ModelIdError,
    },
    /// A registration's ID differs from the ID stored in its metadata.
    #[error("model registration ID {registration_id:?} does not match metadata ID {metadata_id:?} for {registration}")]
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
    /// Two registrations describe the same Rust type identity.
    #[error("duplicate model type identity {identity:?}: first {first}; second {second}")]
    DuplicateIdentity {
        /// The duplicated runtime type identity.
        identity: TypeIdentity,
        /// The deterministically first conflicting registration.
        first: &'static ModelRegistration,
        /// The deterministically second conflicting registration.
        second: &'static ModelRegistration,
    },
}
