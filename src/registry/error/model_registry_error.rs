// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shareable errors produced while building model registries.

use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

use qubit_reflect::capability::CapabilityError;
use qubit_reflect::error::ContextError;
#[cfg(feature = "generic")]
use qubit_reflect::error::LookupError;

use super::super::internal::cause::Cause;
use super::super::model_entry::ModelRegistryOrigin;
use super::ModelRegistryErrorKind;
use crate::metadata::AbiViolation;
use crate::metadata::ModelId;

/// A shareable model-registry construction error retaining its original cause.
///
/// # Examples
///
/// ```
/// use std::error::Error;
/// use qubit_model_metadata::registry::ModelRegistryError;
///
/// fn inspect_registry_failure(error: &ModelRegistryError) {
///     let category = error.kind();
///     let source = error.source();
///     let _ = (category, source);
/// }
/// ```
///
/// A failed public model-registry operation supplies this error value.
#[must_use = "handle model registry failures"]
#[derive(Clone, Debug)]
pub struct ModelRegistryError {
    /// Stable category used for matching this failure.
    kind: ModelRegistryErrorKind,
    /// Model identity associated with the failure, when one is known.
    model_id: Option<ModelId>,
    /// Registration coordinates retained to explain conflicting declarations.
    sources: Vec<ModelRegistryOrigin>,
    /// Original underlying failure, when the registry operation had one.
    cause: Option<Cause>,
}

impl ModelRegistryError {
    /// Wraps a reflection-context failure without model identity coordinates.
    pub(crate) fn context(error: ContextError) -> Self {
        Self {
            kind: ModelRegistryErrorKind::Context,
            model_id: None,
            sources: Vec::new(),
            cause: Some(Cause::Context(error)),
        }
    }

    /// Wraps a capability-resolution failure without model identity
    /// coordinates.
    pub(crate) fn capability(error: CapabilityError) -> Self {
        Self {
            kind: ModelRegistryErrorKind::Capability,
            model_id: None,
            sources: Vec::new(),
            cause: Some(Cause::Capability(error)),
        }
    }

    /// Wraps a generic definition-lookup failure.
    #[cfg(feature = "generic")]
    pub(crate) fn lookup(error: LookupError) -> Self {
        Self {
            kind: ModelRegistryErrorKind::Lookup,
            model_id: None,
            sources: Vec::new(),
            cause: Some(Cause::Lookup(error)),
        }
    }

    /// Retains the model identity, source coordinates, and ABI validation
    /// cause.
    pub(crate) fn invalid_abi(
        model_id: Option<ModelId>,
        sources: Vec<ModelRegistryOrigin>,
        cause: AbiViolation,
    ) -> Self {
        Self {
            kind: ModelRegistryErrorKind::InvalidMetadata,
            model_id,
            sources,
            cause: Some(Cause::Abi(cause)),
        }
    }

    /// Records a registration conflict without an underlying error cause.
    pub(crate) fn conflict(model_id: Option<ModelId>, sources: Vec<ModelRegistryOrigin>) -> Self {
        Self {
            kind: ModelRegistryErrorKind::RegistrationConflict,
            model_id,
            sources,
            cause: None,
        }
    }

    /// Records duplicate use of a stable model identifier.
    pub(crate) fn duplicate(model_id: ModelId, sources: Vec<ModelRegistryOrigin>) -> Self {
        Self {
            kind: ModelRegistryErrorKind::DuplicateModelId,
            model_id: Some(model_id),
            sources,
            cause: None,
        }
    }

    /// Returns the original ABI validation failure, when present.
    ///
    /// # Returns
    ///
    /// The retained ABI failure, or `None` for other registry error kinds.
    #[inline]
    pub fn abi_cause(&self) -> Option<&AbiViolation> {
        match self.cause.as_ref() {
            Some(Cause::Abi(error)) => Some(error),
            _ => None,
        }
    }

    /// Returns the structured failure category.
    ///
    /// # Returns
    ///
    /// The category assigned when the registry error was created.
    #[inline]
    pub const fn kind(&self) -> ModelRegistryErrorKind {
        self.kind
    }

    /// Returns the model identifier associated with the failure.
    ///
    /// # Returns
    ///
    /// The associated stable model ID, or `None` when the failure has no ID.
    #[inline]
    pub const fn model_id(&self) -> Option<ModelId> {
        self.model_id
    }

    /// Returns owned source coordinates associated with the failure.
    ///
    /// # Returns
    ///
    /// The source coordinates retained by this failure; the slice is borrowed
    /// from `self` and remains valid for the duration of that borrow.
    #[must_use]
    #[inline]
    pub fn sources(&self) -> &[ModelRegistryOrigin] {
        &self.sources
    }
}

impl Display for ModelRegistryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "model registry {:?}", self.kind)?;
        if let Some(id) = self.model_id {
            write!(f, " for {}", id.as_str())?;
        }
        if let Some(cause) = &self.cause {
            write!(f, ": {cause}")?;
        }
        Ok(())
    }
}

impl Error for ModelRegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self.cause.as_ref()? {
            Cause::Context(error) => Some(error),
            Cause::Capability(error) => Some(error),
            #[cfg(feature = "generic")]
            Cause::Lookup(error) => Some(error),
            Cause::Abi(error) => Some(error),
        }
    }
}
