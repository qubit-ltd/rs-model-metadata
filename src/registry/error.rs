// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Deterministic model-registry construction errors.

use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result;
use std::any::TypeId;
use std::error::Error;

use qubit_reflect::capability::CapabilityAccessError;
use qubit_reflect::capability::CapabilityOrigin;
use qubit_reflect::error::RegistryError;
use qubit_reflect::identity::CapabilityId;
use qubit_reflect::identity::FragmentIdentity;
use qubit_reflect::registry::CapabilityTarget;

use crate::metadata::AbiViolation;
use crate::metadata::ModelId;

/// Machine-readable registry failure class.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::registry::ModelRegistryErrorKind;
///
/// let kind = ModelRegistryErrorKind::DuplicateModelId;
/// assert_eq!(kind, ModelRegistryErrorKind::DuplicateModelId);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRegistryErrorKind {
    /// A concrete descriptor's intrinsic capabilities could not be resolved.
    CapabilityResolution,
    /// A model capability declares no executable metadata provider.
    FactOnlyCapability,
    /// A model capability declares an incompatible adapter contract.
    AdapterTypeMismatch,
    /// A model metadata capability targets a type or definition outside the
    /// reflected member set.
    UnregisteredModelTarget,
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
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::registry::ModelRegistryError;
///
/// fn describe(error: &ModelRegistryError) -> String {
///     error.to_string()
/// }
/// ```
#[must_use]
#[derive(Clone, Debug)]
pub struct ModelRegistryError {
    /// The machine-readable class of registry construction failure.
    kind: ModelRegistryErrorKind,
    /// The involved stable model ID, when the failure identifies one.
    model_id: Option<ModelId>,
    /// Fragment identities involved in the failure.
    sources: Vec<FragmentIdentity>,
    /// Capability origins involved in the failure.
    origins: Vec<CapabilityOrigin>,
    /// The underlying reflection failure, when reflection initialization
    /// failed.
    reflection: Option<RegistryError>,
    /// Original checked metadata ABI failure, when present.
    abi: Option<AbiViolation>,
    /// Complete intrinsic capability conflict, when present.
    capability: Option<CapabilityAccessError>,
    /// Stable capability identity involved in a provider contract failure.
    capability_id: Option<CapabilityId>,
    /// Capability target rejected by the model projection audit.
    capability_target: Option<CapabilityTarget>,
    /// Adapter type expected by the model metadata capability key.
    expected_adapter_type: Option<TypeId>,
    /// Adapter type declared by the reflected capability descriptor.
    actual_adapter_type: Option<TypeId>,
}

impl ModelRegistryError {
    /// Records a model capability fact without an executable provider.
    ///
    /// # Parameters
    ///
    /// - `capability_id`: stable identity of the fact-only capability.
    /// - `origin`: registration source retained for diagnostics.
    ///
    /// # Returns
    ///
    /// An error retaining the capability ID and origin.
    pub(crate) fn fact_only_capability(capability_id: CapabilityId, origin: CapabilityOrigin) -> Self {
        Self {
            kind: ModelRegistryErrorKind::FactOnlyCapability,
            model_id: None,
            sources: match &origin {
                CapabilityOrigin::Registered { source } => vec![source.clone()],
                CapabilityOrigin::Intrinsic { .. } => Vec::new(),
            },
            origins: vec![origin],
            reflection: None,
            abi: None,
            capability: None,
            capability_id: Some(capability_id),
            capability_target: None,
            expected_adapter_type: None,
            actual_adapter_type: None,
        }
    }

    /// Records a model capability whose adapter contract has the wrong type.
    ///
    /// # Parameters
    ///
    /// - `capability_id`: stable identity of the capability.
    /// - `expected`: adapter type required by the model contract.
    /// - `actual`: adapter type stored in the descriptor.
    /// - `origin`: registration source retained for diagnostics.
    ///
    /// # Returns
    ///
    /// An error retaining both adapter types and the capability origin.
    pub(crate) fn adapter_type_mismatch(
        capability_id: CapabilityId,
        expected: TypeId,
        actual: TypeId,
        origin: CapabilityOrigin,
    ) -> Self {
        Self {
            kind: ModelRegistryErrorKind::AdapterTypeMismatch,
            model_id: None,
            sources: match &origin {
                CapabilityOrigin::Registered { source } => vec![source.clone()],
                CapabilityOrigin::Intrinsic { .. } => Vec::new(),
            },
            origins: vec![origin],
            reflection: None,
            abi: None,
            capability: None,
            capability_id: Some(capability_id),
            capability_target: None,
            expected_adapter_type: Some(expected),
            actual_adapter_type: Some(actual),
        }
    }

    /// Wraps a failure from reflection registry initialization.
    ///
    /// # Parameters
    ///
    /// - `error`: reflection failure whose target and fragment context are
    ///   copied.
    ///
    /// # Returns
    ///
    /// A registry error retaining the reflection error as its source.
    pub(crate) fn reflection(error: RegistryError) -> Self {
        let capability_target = error.capability_target();
        let sources = error.conflicting_fragments().map_or_else(
            || error.fragment_identity().into_iter().cloned().collect(),
            |(left, right)| vec![left.clone(), right.clone()],
        );
        let origins = sources
            .iter()
            .cloned()
            .map(|source| CapabilityOrigin::Registered { source })
            .collect();
        Self {
            kind: ModelRegistryErrorKind::ReflectionRegistry,
            model_id: None,
            sources,
            origins,
            reflection: Some(error),
            abi: None,
            capability: None,
            capability_id: None,
            capability_target,
            expected_adapter_type: None,
            actual_adapter_type: None,
        }
    }

    /// Records registrations that reuse the same stable model ID.
    ///
    /// # Parameters
    ///
    /// - `model_id`: duplicated stable model identifier.
    /// - `sources`: registrations that declared the identifier.
    ///
    /// # Returns
    ///
    /// An error retaining the model ID, sources, and their registered origins.
    pub(crate) fn duplicate(model_id: ModelId, sources: Vec<FragmentIdentity>) -> Self {
        let origins = sources
            .iter()
            .cloned()
            .map(|source| CapabilityOrigin::Registered { source })
            .collect();
        Self {
            kind: ModelRegistryErrorKind::DuplicateModelId,
            model_id: Some(model_id),
            sources,
            origins,
            reflection: None,
            abi: None,
            capability: None,
            capability_id: None,
            capability_target: None,
            expected_adapter_type: None,
            actual_adapter_type: None,
        }
    }

    /// Records a registration whose metadata conflicts with its target.
    ///
    /// # Parameters
    ///
    /// - `model_id`: stable identifier when one is available.
    /// - `sources`: registration fragments that conflict.
    ///
    /// # Returns
    ///
    /// An error retaining the optional identifier and conflicting sources.
    pub(crate) fn conflict(model_id: Option<ModelId>, sources: Vec<FragmentIdentity>) -> Self {
        let origins = sources
            .iter()
            .cloned()
            .map(|source| CapabilityOrigin::Registered { source })
            .collect();
        Self {
            kind: ModelRegistryErrorKind::RegistrationConflict,
            model_id,
            sources,
            origins,
            reflection: None,
            abi: None,
            capability: None,
            capability_id: None,
            capability_target: None,
            expected_adapter_type: None,
            actual_adapter_type: None,
        }
    }

    /// Records a capability targeting a type outside the registered model set.
    ///
    /// The target, capability ID, and originating fragment are retained so
    /// callers can identify which registration failed the projection audit.
    ///
    /// # Parameters
    ///
    /// - `target`: reflected type or definition omitted from the model set.
    /// - `capability_id`: stable identity of the rejected capability.
    /// - `source`: registration fragment that supplied the capability.
    ///
    /// # Returns
    ///
    /// An error retaining the target and complete registration context.
    pub(crate) fn unregistered_model_target(
        target: CapabilityTarget,
        capability_id: CapabilityId,
        source: FragmentIdentity,
    ) -> Self {
        Self {
            kind: ModelRegistryErrorKind::UnregisteredModelTarget,
            model_id: None,
            sources: vec![source.clone()],
            origins: vec![CapabilityOrigin::Registered { source }],
            reflection: None,
            abi: None,
            capability: None,
            capability_id: Some(capability_id),
            capability_target: Some(target),
            expected_adapter_type: None,
            actual_adapter_type: None,
        }
    }

    /// Retains checked metadata failure together with its registration context.
    ///
    /// # Parameters
    ///
    /// - `model_id`: stable model identifier when known.
    /// - `sources`: registration fragments associated with the failure.
    /// - `cause`: checked metadata ABI violation.
    ///
    /// # Returns
    ///
    /// A registration conflict that also retains `cause` as its ABI detail.
    pub(crate) fn invalid_abi(model_id: Option<ModelId>, sources: Vec<FragmentIdentity>, cause: AbiViolation) -> Self {
        let mut error = Self::conflict(model_id, sources);
        error.abi = Some(cause);
        error
    }

    /// Returns the original checked ABI failure, when metadata was malformed.
    ///
    /// # Returns
    ///
    /// The retained ABI violation, or `None` when the error has no ABI cause.
    #[must_use]
    #[inline]
    pub const fn abi_cause(&self) -> Option<&AbiViolation> {
        self.abi.as_ref()
    }

    /// Returns the machine-readable error class.
    ///
    /// # Returns
    ///
    /// The registry failure category used for structured handling.
    #[must_use]
    #[inline]
    pub const fn kind(&self) -> ModelRegistryErrorKind {
        self.kind
    }
    /// Returns the conflicting model ID, or `None` when the failure is not
    /// associated with a model.
    ///
    /// # Returns
    ///
    /// The stable model ID, or `None` when no single model is identified.
    #[must_use]
    #[inline]
    pub const fn model_id(&self) -> Option<ModelId> {
        self.model_id
    }
    /// Returns the capability ID involved in a provider contract failure.
    ///
    /// # Returns
    ///
    /// The capability ID, or `None` when the failure has no capability
    /// identity.
    #[must_use]
    #[inline]
    pub const fn capability_id(&self) -> Option<CapabilityId> {
        self.capability_id
    }
    /// Returns the unregistered target involved in a model projection error.
    ///
    /// # Returns
    ///
    /// The rejected target, or `None` for other registry error categories.
    #[must_use]
    #[inline]
    pub const fn capability_target(&self) -> Option<CapabilityTarget> {
        self.capability_target
    }
    /// Returns the expected adapter type for a provider type mismatch.
    ///
    /// # Returns
    ///
    /// The required adapter `TypeId`, or `None` when no adapter mismatch
    /// occurred.
    #[must_use]
    #[inline]
    pub const fn expected_adapter_type(&self) -> Option<TypeId> {
        self.expected_adapter_type
    }
    /// Returns the actual adapter type for a provider type mismatch.
    ///
    /// # Returns
    ///
    /// The descriptor's adapter `TypeId`, or `None` when no mismatch occurred.
    #[must_use]
    #[inline]
    pub const fn actual_adapter_type(&self) -> Option<TypeId> {
        self.actual_adapter_type
    }
    /// Returns the registration sources involved in the error.
    ///
    /// # Returns
    ///
    /// A slice borrowing the fragment identities retained by this error.
    #[must_use]
    #[inline]
    pub fn sources(&self) -> &[FragmentIdentity] {
        &self.sources
    }

    /// Returns capability origins involved in the error.
    ///
    /// # Returns
    ///
    /// A slice borrowing the origins associated with the failed registrations.
    #[must_use]
    #[inline]
    pub fn origins(&self) -> &[CapabilityOrigin] {
        &self.origins
    }
}

impl Display for ModelRegistryError {
    /// Formats the registry failure with its primary diagnostic context.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        match self.kind {
            ModelRegistryErrorKind::CapabilityResolution => write!(
                formatter,
                "model capability resolution failed: {}",
                self.capability.as_ref().expect("capability cause retained")
            ),
            ModelRegistryErrorKind::FactOnlyCapability => write!(
                formatter,
                "model capability {} has no executable adapter",
                self.capability_id.expect("fact-only errors retain their ID"),
            ),
            ModelRegistryErrorKind::AdapterTypeMismatch => write!(
                formatter,
                "model capability {} adapter type mismatch: expected {:?}, actual {:?}",
                self.capability_id.expect("adapter mismatch errors retain their ID"),
                self.expected_adapter_type
                    .expect("adapter mismatch errors retain the expected type"),
                self.actual_adapter_type
                    .expect("adapter mismatch errors retain the actual type"),
            ),
            ModelRegistryErrorKind::UnregisteredModelTarget => write!(
                formatter,
                "model capability {} targets an unregistered {:?}",
                self.capability_id
                    .expect("unregistered target errors retain capability ID"),
                self.capability_target
                    .expect("unregistered target errors retain target"),
            ),
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
            ModelRegistryErrorKind::RegistrationConflict => {
                match self.model_id {
                    Some(model_id) => write!(formatter, "model capability conflict for {}", model_id.as_str())?,
                    None => formatter.write_str("model capability conflict without a stable model ID")?,
                }
                if let Some(cause) = &self.abi {
                    write!(formatter, ": {cause}")?;
                }
                Ok(())
            }
            ModelRegistryErrorKind::UnsupportedPlatform => formatter.write_str("model registration is unsupported"),
        }
    }
}

impl Error for ModelRegistryError {
    /// Returns the retained ABI, reflection, or capability error as the cause.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.abi
            .as_ref()
            .map(|error| error as &(dyn Error + 'static))
            .or_else(|| self.reflection.as_ref().map(|error| error as &(dyn Error + 'static)))
            .or_else(|| self.capability.as_ref().map(|error| error as &(dyn Error + 'static)))
    }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;
    use std::error::Error;

    use qubit_reflect::RegistryError;
    use qubit_reflect::TypeDefinitionId;
    use qubit_reflect::capability::CapabilityOrigin;
    use qubit_reflect::identity::CapabilityId;
    use qubit_reflect::identity::FragmentIdentity;
    use qubit_reflect::registry::CapabilityTarget;

    use super::ModelRegistryError;
    use super::ModelRegistryErrorKind;

    #[test]
    fn test_unregistered_model_targets_retain_target_and_capability_source() {
        let id = CapabilityId::new("qubit.model.metadata.v1").expect("valid capability ID");
        let source = FragmentIdentity::new("fixture", "tests", 9, 1, "capability", 9);
        for target in [
            CapabilityTarget::Type(TypeId::of::<u8>()),
            CapabilityTarget::TypeDefinition(TypeDefinitionId::of::<u16>()),
        ] {
            let error = ModelRegistryError::unregistered_model_target(target, id, source.clone());
            assert_eq!(error.kind(), ModelRegistryErrorKind::UnregisteredModelTarget);
            assert_eq!(error.capability_target(), Some(target));
            assert_eq!(error.capability_id(), Some(id));
            assert_eq!(error.model_id(), None);
            assert_eq!(error.sources(), std::slice::from_ref(&source));
            assert_eq!(
                error.origins(),
                &[CapabilityOrigin::Registered { source: source.clone() }]
            );
            assert!(error.to_string().contains("qubit.model.metadata.v1"));
            assert!(error.to_string().contains(&format!("{target:?}")));
            assert!(error.source().is_none());
        }
    }

    #[test]
    fn test_provider_contract_errors_retain_machine_readable_context() {
        let id = CapabilityId::new("example.capability").expect("valid capability ID");
        let origin = CapabilityOrigin::Intrinsic {
            type_id: TypeId::of::<u8>(),
        };
        let fact = ModelRegistryError::fact_only_capability(id, origin.clone());
        assert_eq!(fact.kind(), ModelRegistryErrorKind::FactOnlyCapability);
        assert_eq!(fact.capability_id(), Some(id));
        assert_eq!(fact.actual_adapter_type(), None);
        assert_eq!(fact.origins().len(), 1);
        assert!(fact.to_string().contains("no executable adapter"));

        let mismatch = ModelRegistryError::adapter_type_mismatch(id, TypeId::of::<u8>(), TypeId::of::<u16>(), origin);
        assert_eq!(mismatch.kind(), ModelRegistryErrorKind::AdapterTypeMismatch);
        assert_eq!(mismatch.expected_adapter_type(), Some(TypeId::of::<u8>()));
        assert_eq!(mismatch.actual_adapter_type(), Some(TypeId::of::<u16>()));
        assert!(mismatch.to_string().contains("adapter type mismatch"));
    }

    #[test]
    fn test_reflection_failures_keep_registry_context() {
        let cause = RegistryError::unsupported_platform();
        let error = ModelRegistryError::reflection(cause);
        assert_eq!(error.kind(), ModelRegistryErrorKind::ReflectionRegistry);
        assert!(error.source().is_some());
        assert!(error.to_string().contains("reflection registry initialization failed"));
    }
}
