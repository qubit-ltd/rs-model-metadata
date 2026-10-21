// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Deterministic model-registry construction errors.

use std::error::Error;

use qubit_reflect::capability::CapabilityError;
use qubit_reflect::error::ContextError;
#[cfg(feature = "generic")]
use qubit_reflect::error::LookupError;

use super::model_entry::ModelRegistryOrigin;
use crate::metadata::AbiViolation;
use crate::metadata::ModelId;

/// Machine-readable registry failure class.
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

#[derive(Clone, Debug)]
enum Cause {
    Context(ContextError),
    Capability(CapabilityError),
    #[cfg(feature = "generic")]
    Lookup(LookupError),
    Abi(AbiViolation),
}

impl std::fmt::Display for Cause {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Context(error) => std::fmt::Display::fmt(error, formatter),
            Self::Capability(error) => std::fmt::Display::fmt(error, formatter),
            #[cfg(feature = "generic")]
            Self::Lookup(error) => std::fmt::Display::fmt(error, formatter),
            Self::Abi(error) => std::fmt::Display::fmt(error, formatter),
        }
    }
}

/// A shareable model-registry construction error retaining its original cause.
#[derive(Clone, Debug)]
pub struct ModelRegistryError {
    kind: ModelRegistryErrorKind,
    model_id: Option<ModelId>,
    sources: Vec<ModelRegistryOrigin>,
    cause: Option<Cause>,
}

impl ModelRegistryError {
    pub(crate) fn context(error: ContextError) -> Self {
        Self {
            kind: ModelRegistryErrorKind::Context,
            model_id: None,
            sources: Vec::new(),
            cause: Some(Cause::Context(error)),
        }
    }

    pub(crate) fn capability(error: CapabilityError) -> Self {
        Self {
            kind: ModelRegistryErrorKind::Capability,
            model_id: None,
            sources: Vec::new(),
            cause: Some(Cause::Capability(error)),
        }
    }

    #[cfg(feature = "generic")]
    pub(crate) fn lookup(error: LookupError) -> Self {
        Self {
            kind: ModelRegistryErrorKind::Lookup,
            model_id: None,
            sources: Vec::new(),
            cause: Some(Cause::Lookup(error)),
        }
    }

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

    pub(crate) fn conflict(model_id: Option<ModelId>, sources: Vec<ModelRegistryOrigin>) -> Self {
        Self {
            kind: ModelRegistryErrorKind::RegistrationConflict,
            model_id,
            sources,
            cause: None,
        }
    }

    pub(crate) fn duplicate(model_id: ModelId, sources: Vec<ModelRegistryOrigin>) -> Self {
        Self {
            kind: ModelRegistryErrorKind::DuplicateModelId,
            model_id: Some(model_id),
            sources,
            cause: None,
        }
    }

    /// Returns the original ABI validation failure, when present.
    pub fn abi_cause(&self) -> Option<&AbiViolation> {
        match self.cause.as_ref() {
            Some(Cause::Abi(error)) => Some(error),
            _ => None,
        }
    }

    /// Returns the structured failure category.
    pub const fn kind(&self) -> ModelRegistryErrorKind {
        self.kind
    }

    /// Returns the model identifier associated with the failure.
    pub const fn model_id(&self) -> Option<ModelId> {
        self.model_id
    }

    /// Returns owned source coordinates associated with the failure.
    pub fn sources(&self) -> &[ModelRegistryOrigin] {
        &self.sources
    }
}

impl std::fmt::Display for ModelRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

#[cfg(test)]
mod tests {
    use std::any::TypeId;
    use std::error::Error;

    use qubit_reflect::ReflectionContext;

    use super::super::model_entry::ModelRegistryOrigin;
    use super::ModelRegistryError;
    use super::ModelRegistryErrorKind;
    use crate::abi_violation::AbiViolation;
    use crate::metadata::ModelId;

    #[qubit_reflect::Reflect]
    struct Unmodeled;

    fn origin() -> ModelRegistryOrigin {
        ModelRegistryOrigin {
            crate_name: "test-crate",
            crate_version: Some("1.2.3"),
            module_path: "tests::registry_error",
            file: Some("registry_error.rs"),
            line: Some(17),
            column: Some(9),
        }
    }

    #[test]
    fn capability_failure_retains_its_source() {
        let context = ReflectionContext::builder()
            .include_type::<Unmodeled>()
            .build()
            .expect("valid context");
        let descriptor = context.type_by_id(TypeId::of::<Unmodeled>()).expect("registered type");
        let cause = match descriptor.capability::<crate::reflect_facade::ModelMetadataCapability>() {
            Ok(_) => panic!("capability is absent"),
            Err(error) => error,
        };
        let error = ModelRegistryError::capability(cause);

        assert_eq!(error.kind(), ModelRegistryErrorKind::Capability);
        assert!(error.source().is_some());
        assert!(error.to_string().contains("Capability"));
    }

    #[test]
    fn abi_failure_retains_identity_sources_and_original_cause() {
        let model_id = ModelId::new("example.InvalidAbi");
        let source = origin();
        let cause = AbiViolation::new("abi.version", "unsupported ABI version");
        let error = ModelRegistryError::invalid_abi(Some(model_id), vec![source.clone()], cause);

        assert_eq!(error.kind(), ModelRegistryErrorKind::InvalidMetadata);
        assert_eq!(error.model_id(), Some(model_id));
        assert_eq!(error.sources(), &[source]);
        assert_eq!(error.abi_cause(), Some(&cause));
        assert_eq!(
            error.source().expect("original ABI cause").to_string(),
            cause.to_string()
        );
        assert!(error.to_string().contains("example.InvalidAbi"));
    }

    #[test]
    fn cause_free_failures_keep_their_kind_and_source_coordinates() {
        let model_id = ModelId::new("example.Conflict");
        let source = origin();
        let conflict = ModelRegistryError::conflict(Some(model_id), vec![source.clone()]);
        assert_eq!(conflict.kind(), ModelRegistryErrorKind::RegistrationConflict);
        assert_eq!(conflict.model_id(), Some(model_id));
        assert_eq!(conflict.sources(), std::slice::from_ref(&source));
        assert!(conflict.source().is_none());
        assert!(conflict.abi_cause().is_none());

        let duplicate = ModelRegistryError::duplicate(model_id, vec![source.clone()]);
        assert_eq!(duplicate.kind(), ModelRegistryErrorKind::DuplicateModelId);
        assert_eq!(duplicate.model_id(), Some(model_id));
        assert_eq!(duplicate.sources(), &[source]);
        assert!(duplicate.source().is_none());
        assert!(duplicate.to_string().contains("example.Conflict"));
    }
}
