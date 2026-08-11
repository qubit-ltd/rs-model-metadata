// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Immutable lookup indexes over statically linked model registrations.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::LazyLock;

use super::model_registry_error::ModelRegistryError;
use crate::metadata_resolver::MetadataResolver;
use crate::model_id::ModelId;
use crate::model_registration::MODEL_REGISTRATIONS;
use crate::model_registration::ModelRegistration;
use crate::type_metadata::TypeIdentity;
use crate::type_metadata::TypeMetadata;

/// An immutable lookup index over linked model registrations.
#[derive(Debug)]
pub struct ModelRegistry {
    /// Registrations ordered by stable ID and deterministic tie-breakers.
    registrations: Vec<&'static ModelRegistration>,
    /// Positions in `registrations`, indexed by stable model ID.
    indices: BTreeMap<ModelId, usize>,
    /// Positions in `registrations`, indexed by Rust runtime type identity.
    identity_indices: HashMap<TypeIdentity, usize>,
}

impl ModelRegistry {
    /// Builds an immutable registry from static model registrations.
    ///
    /// Registrations are copied and sorted deterministically. Returns
    /// [`ModelRegistryError`] when a registration disagrees with its metadata
    /// ID or when two registrations share the same stable ID.
    pub fn from_registrations(
        registrations: impl IntoIterator<Item = &'static ModelRegistration>,
    ) -> Result<Self, ModelRegistryError> {
        let mut registrations: Vec<_> = registrations.into_iter().collect();
        registrations.sort_unstable_by(compare_registrations);

        for registration in &registrations {
            let registration_id = registration.id();
            let metadata_id = registration.metadata().id();
            ModelId::validate(registration_id.as_str()).map_err(|source| {
                ModelRegistryError::InvalidRegistrationId {
                    registration,
                    id: registration_id,
                    source,
                }
            })?;
            ModelId::validate(metadata_id.as_str()).map_err(|source| {
                ModelRegistryError::InvalidMetadataId {
                    registration,
                    id: metadata_id,
                    source,
                }
            })?;
            if registration_id != metadata_id {
                return Err(ModelRegistryError::MetadataIdMismatch {
                    registration,
                    registration_id,
                    metadata_id,
                });
            }
        }

        for pair in registrations.windows(2) {
            let [first, second] = pair else {
                unreachable!(
                    "two-item windows always destructure to two elements"
                );
            };
            if first.id() == second.id() {
                return Err(ModelRegistryError::DuplicateId {
                    id: first.id(),
                    first,
                    second,
                });
            }
        }

        let mut indices = BTreeMap::new();
        let mut identity_indices = HashMap::new();
        for (index, registration) in registrations.iter().enumerate() {
            indices.insert(registration.id(), index);
            if let Some(first_index) = identity_indices
                .insert(registration.metadata().identity(), index)
            {
                return Err(ModelRegistryError::DuplicateIdentity {
                    identity: registration.metadata().identity(),
                    first: registrations[first_index],
                    second: registration,
                });
            }
        }
        Ok(Self {
            registrations,
            indices,
            identity_indices,
        })
    }

    /// Returns the lazily constructed registry for all linked model crates.
    ///
    /// Returns a shared error reference when the linked registrations are
    /// invalid. It never panics and initializes the registry at most once.
    pub fn try_global() -> Result<&'static Self, &'static ModelRegistryError> {
        GLOBAL_MODEL_REGISTRY.as_ref()
    }

    /// Returns the lazily constructed registry for all linked model crates.
    ///
    /// # Panics
    ///
    /// Panics when linked model registrations contain an invalid or duplicate
    /// stable ID. Use [`ModelRegistry::try_global`] when callers need to handle
    /// configuration errors without panicking.
    #[must_use]
    pub fn global() -> &'static Self {
        Self::try_global().unwrap_or_else(|error| {
            panic!("invalid global model registry: {error}");
        })
    }

    /// Returns metadata for `id`, or `None` if this registry has no such model.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&'static TypeMetadata> {
        self.indices
            .get(id)
            .map(|index| self.registrations[*index].metadata())
    }

    /// Returns the registration for `id`, or `None` if this registry has no
    /// such model.
    #[must_use]
    pub fn registration(&self, id: &str) -> Option<&'static ModelRegistration> {
        self.indices.get(id).map(|index| self.registrations[*index])
    }

    /// Returns all registrations in deterministic stable-ID order.
    #[must_use]
    pub fn registrations(
        &self,
    ) -> impl ExactSizeIterator<Item = &'static ModelRegistration> + '_ {
        self.registrations.iter().copied()
    }
}

impl MetadataResolver for ModelRegistry {
    /// Resolves a runtime type identity from this immutable model collection.
    fn resolve(&self, identity: TypeIdentity) -> Option<&'static TypeMetadata> {
        self.identity_indices
            .get(&identity)
            .map(|index| self.registrations[*index].metadata())
    }
}

/// Compares registrations using the deterministic registry ordering.
fn compare_registrations(
    left: &&'static ModelRegistration,
    right: &&'static ModelRegistration,
) -> std::cmp::Ordering {
    (
        left.id(),
        left.rust_type_name(),
        left.rust_module_path(),
        left.source().file(),
        left.source().line(),
        left.source().column(),
    )
        .cmp(&(
            right.id(),
            right.rust_type_name(),
            right.rust_module_path(),
            right.source().file(),
            right.source().line(),
            right.source().column(),
        ))
}

/// Builds the global registry from the registrations linked into this binary.
static GLOBAL_MODEL_REGISTRY: LazyLock<
    Result<ModelRegistry, ModelRegistryError>,
> = LazyLock::new(|| {
    ModelRegistry::from_registrations(MODEL_REGISTRATIONS.iter())
});
