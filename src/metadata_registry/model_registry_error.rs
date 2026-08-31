// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Deterministic model-registry construction errors.
// qubit-style: allow multiple-public-types

use qubit_reflect::error::RegistryError;
use qubit_reflect::identity::FragmentIdentity;

use crate::ModelId;

/// Machine-readable registry failure class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRegistryErrorKind {
    /// The shared reflection registry could not initialize.
    ReflectionRegistry,
    /// Two linked registrations declared the same model ID.
    DuplicateModelId,
    /// A registration does not match its target metadata.
    RegistrationConflict,
    /// Registry construction is unavailable on this platform.
    UnsupportedPlatform,
}

/// A shareable model-registry construction error.
#[derive(Clone, Debug)]
pub struct ModelRegistryError {
    kind: ModelRegistryErrorKind,
    model_id: Option<ModelId>,
    sources: Vec<FragmentIdentity>,
    reflection: Option<RegistryError>,
}

impl ModelRegistryError {
    pub(crate) fn reflection(error: RegistryError) -> Self {
        Self {
            kind: ModelRegistryErrorKind::ReflectionRegistry,
            model_id: None,
            sources: error.fragment_identity().into_iter().cloned().collect(),
            reflection: Some(error),
        }
    }

    pub(crate) fn duplicate(model_id: ModelId, sources: Vec<FragmentIdentity>) -> Self {
        Self {
            kind: ModelRegistryErrorKind::DuplicateModelId,
            model_id: Some(model_id),
            sources,
            reflection: None,
        }
    }

    pub(crate) fn conflict(model_id: ModelId, sources: Vec<FragmentIdentity>) -> Self {
        Self {
            kind: ModelRegistryErrorKind::RegistrationConflict,
            model_id: Some(model_id),
            sources,
            reflection: None,
        }
    }

    /// Returns the machine-readable error class.
    pub const fn kind(&self) -> ModelRegistryErrorKind {
        self.kind
    }
    /// Returns the conflicting model ID, when applicable.
    pub const fn model_id(&self) -> Option<ModelId> {
        self.model_id
    }
    /// Returns the registration sources involved in the error.
    pub fn sources(&self) -> &[FragmentIdentity] {
        &self.sources
    }
}

impl core::fmt::Display for ModelRegistryError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.kind {
            ModelRegistryErrorKind::ReflectionRegistry => write!(
                formatter,
                "reflection registry initialization failed: {}",
                self.reflection.as_ref().expect("reflection errors retain their source"),
            ),
            ModelRegistryErrorKind::DuplicateModelId => write!(
                formatter,
                "duplicate model ID {}",
                self.model_id.expect("duplicate errors retain their ID").as_str(),
            ),
            ModelRegistryErrorKind::RegistrationConflict => write!(
                formatter,
                "model registration conflict for {}",
                self.model_id.expect("conflict errors retain their ID").as_str(),
            ),
            ModelRegistryErrorKind::UnsupportedPlatform => formatter.write_str("model registration is unsupported"),
        }
    }
}

impl std::error::Error for ModelRegistryError {}
