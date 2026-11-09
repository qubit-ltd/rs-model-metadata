// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::EntityMetadata;
use super::ModelMetadata;
use super::ModelRole;
use super::ProjectionMetadata;
use super::ValueMetadata;
use crate::type_metadata::EnumMetadata;

/// Role-specific metadata payload.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::ModelMetadata;
/// use qubit_model_metadata::metadata::ModelRole;
/// use qubit_model_metadata::metadata::RoleMetadata;
///
/// let payload = RoleMetadata::Model(ModelMetadata);
/// assert_eq!(payload.role(), ModelRole::Model);
/// ```
#[derive(Clone, Copy, Debug)]
pub enum RoleMetadata {
    /// Entity-specific payload.
    Entity(
        /// The identifier field and typed accessor associated with the entity.
        EntityMetadata,
    ),
    /// Projection-specific payload.
    Projection(
        /// The projection identifier and its optional declared entity source.
        ProjectionMetadata,
    ),
    /// General model payload.
    Model(
        /// The marker metadata identifying the type as a general model.
        ModelMetadata,
    ),
    /// Enum-specific payload.
    Enum(
        /// The enum's variants and their source-order metadata.
        EnumMetadata,
    ),
    /// Value-specific payload.
    Value(
        /// The value's transparent-field and canonical-codec metadata.
        ValueMetadata,
    ),
}

impl RoleMetadata {
    /// Returns this payload's role discriminator.
    ///
    /// # Returns
    ///
    /// The role corresponding to the active payload variant.
    #[must_use]
    #[inline]
    pub const fn role(&self) -> ModelRole {
        match self {
            Self::Entity(_) => ModelRole::Entity,
            Self::Projection(_) => ModelRole::Projection,
            Self::Model(_) => ModelRole::Model,
            Self::Enum(_) => ModelRole::Enum,
            Self::Value(_) => ModelRole::Value,
        }
    }
}
