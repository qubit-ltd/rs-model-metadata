// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Machine-readable model-registry failure categories.

/// Machine-readable registry failure class.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::registry::{ModelRegistryError, ModelRegistryErrorKind};
///
/// fn inspect_registry_failure(error: &ModelRegistryError) {
///     let kind: ModelRegistryErrorKind = error.kind();
///     match kind {
///         ModelRegistryErrorKind::Context
///         | ModelRegistryErrorKind::Capability
///         | ModelRegistryErrorKind::Lookup
///         | ModelRegistryErrorKind::InvalidMetadata
///         | ModelRegistryErrorKind::DuplicateModelId
///         | ModelRegistryErrorKind::RegistrationConflict => {}
///     }
/// }
/// ```
///
/// The error value is supplied by a failed public model-registry operation.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRegistryErrorKind {
    /// The reflection context could not be queried.
    Context,
    /// A model capability could not be resolved or validated.
    Capability,
    /// A reflected type or definition lookup failed.
    Lookup,
    /// A model metadata value does not match its reflected descriptor.
    InvalidMetadata,
    /// Two declarations reused one stable model identifier.
    DuplicateModelId,
    /// Registrations conflict for the same target.
    RegistrationConflict,
}
