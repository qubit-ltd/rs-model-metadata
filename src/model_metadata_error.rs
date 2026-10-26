// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Failures resolving model semantics for an exact reflected descriptor.

use std::any::TypeId;

use qubit_reflect::capability::CapabilityError;

use crate::metadata::AbiViolation;

/// A capability or ABI failure with the exact queried Rust type identity.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::ModelMetadataError;
///
/// // This error is supplied by `ModelRegistry::metadata_for` when a
/// // descriptor's capability lookup fails.
/// fn is_capability_error(error: &ModelMetadataError) -> bool {
///     matches!(error, ModelMetadataError::Capability { .. })
/// }
/// ```
#[must_use]
#[derive(Clone, Debug, thiserror::Error)]
pub enum ModelMetadataError {
    /// Intrinsic capability declarations conflict before a provider can run.
    #[error("capability resolution failed for {type_name}: {source}")]
    Capability {
        /// Process-local identity of the queried descriptor.
        type_id: TypeId,
        /// Diagnostic Rust name of the queried descriptor.
        type_name: &'static str,
        /// The complete intrinsic declaration conflict.
        #[source]
        source: Box<CapabilityError>,
    },
    /// A provider returned metadata belonging to a different descriptor.
    #[error("metadata ABI validation failed for {type_name}: {source}")]
    Abi {
        /// Process-local identity of the queried descriptor.
        type_id: TypeId,
        /// Diagnostic Rust name of the queried descriptor.
        type_name: &'static str,
        /// The generated metadata contract violation.
        #[source]
        source: AbiViolation,
    },
}
