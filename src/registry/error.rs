// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Deterministic model-registry construction errors.
// qubit-style: allow multiple-public-types

use std::any::TypeId;

#[cfg(feature = "generic")]
use qubit_reflect::TypeDefinitionDescriptor;
use qubit_reflect::capability::CapabilityAccessError;
use qubit_reflect::error::RegistryError;
use qubit_reflect::identity::CapabilityId;
use qubit_reflect::identity::FragmentIdentity;
#[cfg(feature = "generic")]
use qubit_reflect::registry::ReflectRegistry;

use crate::metadata::ModelId;

/// Machine-readable registry failure class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRegistryErrorKind {
    /// A concrete descriptor's intrinsic capabilities could not be resolved.
    CapabilityResolution,
    /// A model capability declares no executable metadata provider.
    FactOnlyCapability,
    /// A model capability declares an incompatible adapter contract.
    AdapterTypeMismatch,
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
#[must_use]
#[derive(Clone, Debug)]
pub struct ModelRegistryError {
    /// The machine-readable class of registry construction failure.
    kind: ModelRegistryErrorKind,
    /// The involved stable model ID, when the failure identifies one.
    model_id: Option<ModelId>,
    /// Fragment identities involved in the failure.
    sources: Vec<FragmentIdentity>,
    /// The underlying reflection failure, when reflection initialization
    /// failed.
    reflection: Option<RegistryError>,
    /// Complete intrinsic capability conflict, when present.
    capability: Option<CapabilityAccessError>,
    /// Stable capability identity involved in a provider contract failure.
    capability_id: Option<CapabilityId>,
    /// Adapter type expected by the model metadata capability key.
    expected_adapter_type: Option<TypeId>,
    /// Adapter type declared by the reflected capability descriptor.
    actual_adapter_type: Option<TypeId>,
}

impl ModelRegistryError {
    /// Retains an intrinsic capability conflict and its registration source.
    pub(crate) fn capability(
        error: CapabilityAccessError,
        source: FragmentIdentity,
    ) -> Self {
        Self {
            kind: ModelRegistryErrorKind::CapabilityResolution,
            model_id: None,
            sources: vec![source],
            reflection: None,
            capability: Some(error),
            capability_id: None,
            expected_adapter_type: None,
            actual_adapter_type: None,
        }
    }

    /// Converts a reflection capability lookup failure into registry context.
    #[cfg(feature = "generic")]
    pub(crate) fn capability_access(
        error: CapabilityAccessError,
        reflection: &ReflectRegistry,
        definition: &TypeDefinitionDescriptor,
    ) -> Self {
        let source = reflection.definition_source(definition.id()).cloned();
        let (kind, capability_id, expected_adapter_type, actual_adapter_type) =
            match &error {
                CapabilityAccessError::FactOnly { id, adapter_type } => (
                    ModelRegistryErrorKind::FactOnlyCapability,
                    Some(*id),
                    None,
                    Some(*adapter_type),
                ),
                CapabilityAccessError::AdapterTypeMismatch {
                    id,
                    expected,
                    actual,
                } => (
                    ModelRegistryErrorKind::AdapterTypeMismatch,
                    Some(*id),
                    Some(*expected),
                    Some(*actual),
                ),
                CapabilityAccessError::IntrinsicConflict(_) => (
                    ModelRegistryErrorKind::CapabilityResolution,
                    None,
                    None,
                    None,
                ),
            };
        Self {
            kind,
            model_id: None,
            sources: source.into_iter().collect(),
            reflection: None,
            capability: Some(error),
            capability_id,
            expected_adapter_type,
            actual_adapter_type,
        }
    }

    /// Records a model capability fact without an executable provider.
    pub(crate) fn fact_only_capability(
        capability_id: CapabilityId,
        source: FragmentIdentity,
    ) -> Self {
        Self {
            kind: ModelRegistryErrorKind::FactOnlyCapability,
            model_id: None,
            sources: vec![source],
            reflection: None,
            capability: None,
            capability_id: Some(capability_id),
            expected_adapter_type: None,
            actual_adapter_type: None,
        }
    }

    /// Records a model capability whose adapter contract has the wrong type.
    pub(crate) fn adapter_type_mismatch(
        capability_id: CapabilityId,
        expected: TypeId,
        actual: TypeId,
        source: FragmentIdentity,
    ) -> Self {
        Self {
            kind: ModelRegistryErrorKind::AdapterTypeMismatch,
            model_id: None,
            sources: vec![source],
            reflection: None,
            capability: None,
            capability_id: Some(capability_id),
            expected_adapter_type: Some(expected),
            actual_adapter_type: Some(actual),
        }
    }

    /// Wraps a failure from reflection registry initialization.
    pub(crate) fn reflection(error: RegistryError) -> Self {
        let sources = error.conflicting_fragments().map_or_else(
            || error.fragment_identity().into_iter().cloned().collect(),
            |(left, right)| vec![left.clone(), right.clone()],
        );
        Self {
            kind: ModelRegistryErrorKind::ReflectionRegistry,
            model_id: None,
            sources,
            reflection: Some(error),
            capability: None,
            capability_id: None,
            expected_adapter_type: None,
            actual_adapter_type: None,
        }
    }

    /// Records registrations that reuse the same stable model ID.
    pub(crate) fn duplicate(
        model_id: ModelId,
        sources: Vec<FragmentIdentity>,
    ) -> Self {
        Self {
            kind: ModelRegistryErrorKind::DuplicateModelId,
            model_id: Some(model_id),
            sources,
            reflection: None,
            capability: None,
            capability_id: None,
            expected_adapter_type: None,
            actual_adapter_type: None,
        }
    }

    /// Records a registration whose metadata conflicts with its target.
    pub(crate) fn conflict(
        model_id: Option<ModelId>,
        sources: Vec<FragmentIdentity>,
    ) -> Self {
        Self {
            kind: ModelRegistryErrorKind::RegistrationConflict,
            model_id,
            sources,
            reflection: None,
            capability: None,
            capability_id: None,
            expected_adapter_type: None,
            actual_adapter_type: None,
        }
    }

    /// Returns the machine-readable error class.
    #[must_use]
    #[inline(always)]
    pub const fn kind(&self) -> ModelRegistryErrorKind {
        self.kind
    }
    /// Returns the conflicting model ID, or `None` when the failure is not
    /// associated with a model.
    #[must_use]
    #[inline(always)]
    pub const fn model_id(&self) -> Option<ModelId> {
        self.model_id
    }
    /// Returns the capability ID involved in a provider contract failure.
    #[must_use]
    #[inline(always)]
    pub const fn capability_id(&self) -> Option<CapabilityId> {
        self.capability_id
    }
    /// Returns the expected adapter type for a provider type mismatch.
    #[must_use]
    #[inline(always)]
    pub const fn expected_adapter_type(&self) -> Option<TypeId> {
        self.expected_adapter_type
    }
    /// Returns the actual adapter type for a provider type mismatch.
    #[must_use]
    #[inline(always)]
    pub const fn actual_adapter_type(&self) -> Option<TypeId> {
        self.actual_adapter_type
    }
    /// Returns the registration sources involved in the error.
    #[must_use]
    #[inline(always)]
    pub fn sources(&self) -> &[FragmentIdentity] {
        &self.sources
    }
}

impl core::fmt::Display for ModelRegistryError {
    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        match self.kind {
            ModelRegistryErrorKind::CapabilityResolution => write!(
                formatter,
                "model capability resolution failed: {}",
                self.capability.as_ref().expect("capability cause retained")
            ),
            ModelRegistryErrorKind::FactOnlyCapability => write!(
                formatter,
                "model capability {} has no executable adapter",
                self.capability_id
                    .expect("fact-only errors retain their ID"),
            ),
            ModelRegistryErrorKind::AdapterTypeMismatch => write!(
                formatter,
                "model capability {} adapter type mismatch: expected {:?}, actual {:?}",
                self.capability_id
                    .expect("adapter mismatch errors retain their ID"),
                self.expected_adapter_type
                    .expect("adapter mismatch errors retain the expected type"),
                self.actual_adapter_type
                    .expect("adapter mismatch errors retain the actual type"),
            ),
            ModelRegistryErrorKind::ReflectionRegistry => write!(
                formatter,
                "reflection registry initialization failed: {}",
                self.reflection
                    .as_ref()
                    .expect("reflection errors retain their source"),
            ),
            ModelRegistryErrorKind::DuplicateModelId => write!(
                formatter,
                "duplicate model ID {}",
                self.model_id
                    .expect("duplicate errors retain their ID")
                    .as_str(),
            ),
            ModelRegistryErrorKind::RegistrationConflict => match self.model_id
            {
                Some(model_id) => write!(
                    formatter,
                    "model capability conflict for {}",
                    model_id.as_str()
                ),
                None => formatter.write_str(
                    "model capability conflict without a stable model ID",
                ),
            },
            ModelRegistryErrorKind::UnsupportedPlatform => {
                formatter.write_str("model registration is unsupported")
            }
        }
    }
}

impl std::error::Error for ModelRegistryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.reflection
            .as_ref()
            .map(|error| error as &(dyn std::error::Error + 'static))
            .or_else(|| {
                self.capability
                    .as_ref()
                    .map(|error| error as &(dyn std::error::Error + 'static))
            })
    }
}
