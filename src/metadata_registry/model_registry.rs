// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Frozen indexes over concrete and generic model registrations.

use std::any::TypeId;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::OnceLock;

use qubit_reflect::registry::ReflectRegistry;

use super::ModelRegistryError;
use crate::GenericModelMetadata;
use crate::ModelId;
use crate::ModelRegistration;
use crate::ModelRegistrationFactory;
use crate::TypeMetadata;

/// An immutable registry sorted by stable model ID and fragment identity.
#[derive(Debug)]
pub struct ModelRegistry {
    /// Registrations in deterministic model-ID and fragment-identity order.
    registrations: Box<[ModelRegistration]>,
    /// Lookup from a stable model ID to a registration index.
    indices: BTreeMap<ModelId, usize>,
    /// Lookup from an exact Rust type identity to a registration index.
    type_indices: HashMap<TypeId, usize>,
    /// Generic definitions retained in deterministic registration order.
    generic_definitions: Box<[&'static GenericModelMetadata]>,
}

impl ModelRegistry {
    /// Projects concrete model registrations from a frozen reflection
    /// snapshot and combines them with model-owned generic templates.
    ///
    /// Concrete source provenance always comes from the reflection type
    /// fragment. Generic templates have no concrete reflection root and keep
    /// their independent model registration.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRegistryError`] for duplicate model IDs or inconsistent
    /// concrete registration metadata.
    #[must_use = "handle invalid model registrations"]
    pub fn from_reflect_registry(
        reflection: &'static ReflectRegistry,
        generic_registrations: impl IntoIterator<Item = ModelRegistration>,
    ) -> Result<Self, ModelRegistryError> {
        let mut registrations = Vec::new();
        for (descriptor, source) in reflection.types_with_identity() {
            let Some(provider) = reflection
                .capabilities(descriptor.type_id())
                .get(crate::reflect_facade::model_metadata_key())
            else {
                continue;
            };
            let metadata = provider();
            metadata.assert_valid_descriptor(descriptor);
            if metadata.model_id().is_some() {
                registrations.push(ModelRegistration::from_concrete(metadata, source));
            }
        }
        registrations.extend(
            generic_registrations
                .into_iter()
                .filter(|registration| registration.generic().is_some()),
        );
        Self::build(registrations)
    }

    /// Builds a deterministic registry from static compatibility registrations.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRegistryError`] when registrations repeat a model ID or
    /// a concrete registration conflicts with its metadata.
    #[must_use = "handle invalid model registrations"]
    pub fn from_registrations(
        registrations: impl IntoIterator<Item = &'static ModelRegistration>,
    ) -> Result<Self, ModelRegistryError> {
        Self::build(registrations.into_iter().copied().collect())
    }

    /// Validates and indexes owned registrations in deterministic order.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRegistryError`] for duplicate model IDs or inconsistent
    /// concrete registration metadata.
    fn build(mut registrations: Vec<ModelRegistration>) -> Result<Self, ModelRegistryError> {
        registrations.sort_by(compare_registrations);
        for pair in registrations.windows(2) {
            if pair[0].model_id() == pair[1].model_id() {
                let sources = pair.iter().map(|registration| registration.source().clone()).collect();
                return Err(ModelRegistryError::duplicate(pair[0].model_id(), sources));
            }
        }

        let mut indices = BTreeMap::new();
        let mut type_indices = HashMap::new();
        let mut generic_definitions = Vec::new();
        for (index, registration) in registrations.iter().enumerate() {
            indices.insert(registration.model_id(), index);
            if let Some(metadata) = registration.metadata() {
                if metadata.model_id() != Some(registration.model_id()) {
                    return Err(ModelRegistryError::conflict(
                        registration.model_id(),
                        vec![registration.source().clone()],
                    ));
                }
                if let Some(previous) = type_indices.insert(metadata.type_id(), index) {
                    return Err(ModelRegistryError::conflict(
                        registration.model_id(),
                        vec![registrations[previous].source().clone(), registration.source().clone()],
                    ));
                }
            }
            if let Some(generic) = registration.generic() {
                generic_definitions.push(generic);
            }
        }

        Ok(Self {
            registrations: registrations.into_boxed_slice(),
            indices,
            type_indices,
            generic_definitions: generic_definitions.into_boxed_slice(),
        })
    }

    /// Initializes reflection first, then freezes all linked model
    /// registrations.
    ///
    /// # Errors
    ///
    /// Returns [`ModelRegistryError`] when reflection initialization or model
    /// registration validation fails. The result is cached for the process.
    #[must_use = "handle model registry initialization failure"]
    pub fn try_global() -> Result<&'static Self, ModelRegistryError> {
        static REGISTRY: OnceLock<Result<ModelRegistry, ModelRegistryError>> = OnceLock::new();
        match REGISTRY.get_or_init(|| {
            let generic_registrations: Vec<_> = inventory::iter::<ModelRegistrationFactory>
                .into_iter()
                .map(|factory| {
                    let ModelRegistrationFactory(factory) = *factory;
                    factory()
                })
                .filter(|registration| registration.generic().is_some())
                .collect();
            Self::from_reflect_registry(
                ReflectRegistry::initialize().map_err(ModelRegistryError::reflection)?,
                generic_registrations,
            )
        }) {
            Ok(registry) => Ok(registry),
            Err(error) => Err(error.clone()),
        }
    }

    /// Returns the process-wide registry or panics with a stable diagnostic.
    ///
    /// # Panics
    ///
    /// Panics when the cached global registry initialization failed.
    #[must_use]
    pub fn global() -> &'static Self {
        Self::try_global().unwrap_or_else(|error| panic!("invalid global model registry: {error}"))
    }

    /// Returns the complete registration for a stable ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&ModelRegistration> {
        if ModelId::validate(id).is_err() {
            return None;
        }
        self.registrations.get(*self.indices.get(id)?)
    }

    /// Returns concrete metadata for a stable ID.
    #[must_use]
    pub fn metadata(&self, id: &str) -> Option<&'static TypeMetadata> {
        self.get(id)?.metadata()
    }

    /// Returns generic-template metadata for a stable ID.
    #[must_use]
    pub fn generic(&self, id: &str) -> Option<&'static GenericModelMetadata> {
        self.get(id)?.generic()
    }

    /// Returns registered concrete metadata by exact Rust identity.
    #[must_use]
    pub fn by_type_id(&self, type_id: TypeId) -> Option<&'static TypeMetadata> {
        self.registrations.get(*self.type_indices.get(&type_id)?)?.metadata()
    }

    /// Returns registrations in deterministic order.
    #[must_use]
    #[inline(always)]
    pub fn registrations(&self) -> &[ModelRegistration] {
        &self.registrations
    }

    /// Returns registered generic definitions in deterministic order.
    #[must_use]
    #[inline(always)]
    pub fn generic_definitions(&self) -> &[&'static GenericModelMetadata] {
        &self.generic_definitions
    }
}

/// Compares registrations by stable model ID and then fragment identity.
fn compare_registrations(left: &ModelRegistration, right: &ModelRegistration) -> std::cmp::Ordering {
    left.model_id()
        .cmp(&right.model_id())
        .then_with(|| left.source().cmp(right.source()))
}
