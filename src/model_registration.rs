// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Distributed registrations for concrete model types and generic templates.
// qubit-style: allow multiple-public-types

use qubit_reflect::identity::FragmentIdentity;

use crate::GenericModelMetadata;
use crate::ModelId;
use crate::ModelRole;
use crate::TypeMetadata;

/// The concrete or generic target declared by one registration.
#[derive(Clone, Copy, Debug)]
pub enum ModelRegistrationTarget {
    /// A concrete model type.
    Concrete(&'static TypeMetadata),
    /// A generic model definition.
    Generic(&'static GenericModelMetadata),
}

/// One statically linked model registration.
#[derive(Clone, Copy, Debug)]
pub struct ModelRegistration {
    /// The stable identifier under which the target is registered.
    model_id: ModelId,
    /// The concrete model or generic model target.
    target: ModelRegistrationTarget,
    /// The fragment that supplied this registration.
    source: &'static FragmentIdentity,
}

impl ModelRegistration {
    /// Creates a concrete registration for generated model metadata.
    ///
    /// # Panics
    ///
    /// Panics when `metadata` has no stable model ID.
    #[doc(hidden)]
    #[must_use = "the stable model ID identifies the registered target"]
    pub(crate) const fn from_concrete(metadata: &'static TypeMetadata, source: &'static FragmentIdentity) -> Self {
        let Some(model_id) = metadata.model_id() else {
            panic!("QMM-ABI-060: registered concrete metadata requires a model ID");
        };
        Self {
            model_id,
            target: ModelRegistrationTarget::Concrete(metadata),
            source,
        }
    }

    /// Creates a generic registration for generated model metadata.
    #[doc(hidden)]
    #[must_use]
    pub(crate) const fn from_generic(
        metadata: &'static GenericModelMetadata,
        source: &'static FragmentIdentity,
    ) -> Self {
        Self {
            model_id: metadata.model_id(),
            target: ModelRegistrationTarget::Generic(metadata),
            source,
        }
    }

    /// Returns the stable identifier under which this target is registered.
    #[must_use = "the stable model ID identifies the registered target"]
    pub const fn model_id(&self) -> ModelId {
        self.model_id
    }

    /// Returns the target model role.
    #[must_use]
    pub const fn role(&self) -> ModelRole {
        match self.target {
            ModelRegistrationTarget::Concrete(value) => value.role(),
            ModelRegistrationTarget::Generic(value) => value.role(),
        }
    }

    /// Returns the concrete or generic registration target.
    #[must_use]
    pub const fn target(&self) -> ModelRegistrationTarget {
        self.target
    }

    /// Returns the source fragment identity.
    #[must_use]
    pub const fn source(&self) -> &'static FragmentIdentity {
        self.source
    }

    /// Returns concrete metadata when this is a concrete registration.
    #[must_use]
    pub const fn metadata(&self) -> Option<&'static TypeMetadata> {
        match self.target {
            ModelRegistrationTarget::Concrete(value) => Some(value),
            _ => None,
        }
    }

    /// Returns generic metadata when this is a generic registration.
    #[must_use]
    pub const fn generic(&self) -> Option<&'static GenericModelMetadata> {
        match self.target {
            ModelRegistrationTarget::Generic(value) => Some(value),
            _ => None,
        }
    }
}

/// A factory submitted by generated code through `inventory`.
pub struct ModelRegistrationFactory(pub fn() -> ModelRegistration);

inventory::collect!(ModelRegistrationFactory);
