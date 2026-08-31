// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

// qubit-style: allow multiple-public-types
//! The five supported model roles and their role-specific payloads.

use crate::CodecMetadata;
use crate::DeclaredEntityTarget;
use crate::EnumMetadata;
use crate::FieldMetadata;

/// The semantic role assigned by a model macro.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ModelRole {
    /// Persisted entity with an identifier.
    Entity,
    /// Open or fixed view over entity data.
    Projection,
    /// General structured model.
    Model,
    /// Enumerated model.
    Enum,
    /// Value object.
    Value,
}

/// Entity-specific metadata.
#[derive(Clone, Copy, Debug)]
pub struct EntityMetadata {
    identifier: &'static FieldMetadata,
}

impl EntityMetadata {
    /// Creates entity metadata.
    #[must_use]
    pub const fn new(identifier: &'static FieldMetadata) -> Self {
        Self { identifier }
    }

    /// Returns the entity identifier field.
    #[must_use]
    pub const fn identifier(&self) -> &'static FieldMetadata {
        self.identifier
    }
}

/// Projection-specific metadata.
#[derive(Clone, Copy, Debug)]
pub struct ProjectionMetadata {
    identifier: &'static FieldMetadata,
    source: Option<&'static DeclaredEntityTarget>,
}

impl ProjectionMetadata {
    /// Creates projection metadata.
    #[must_use]
    pub const fn new(identifier: &'static FieldMetadata, source: Option<&'static DeclaredEntityTarget>) -> Self {
        Self { identifier, source }
    }

    /// Returns the projection identifier field.
    #[must_use]
    pub const fn identifier(&self) -> &'static FieldMetadata {
        self.identifier
    }

    /// Returns the optional declared source without consulting a registry.
    #[must_use]
    pub const fn source(&self) -> Option<&'static DeclaredEntityTarget> {
        self.source
    }

    /// Returns whether undeclared source fields are accepted.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.source.is_none()
    }

    /// Returns whether the projection field set is fixed.
    #[must_use]
    pub const fn is_fixed(&self) -> bool {
        self.source.is_some()
    }
}

/// Model-specific metadata, intentionally empty in the first version.
#[derive(Clone, Copy, Debug, Default)]
pub struct ModelMetadata;

/// Value-specific metadata.
#[derive(Clone, Copy, Debug)]
pub struct ValueMetadata {
    transparent_field: Option<&'static FieldMetadata>,
    canonical_codec: Option<&'static CodecMetadata>,
}

impl ValueMetadata {
    /// Creates value metadata.
    #[must_use]
    pub const fn new(
        transparent_field: Option<&'static FieldMetadata>,
        canonical_codec: Option<&'static CodecMetadata>,
    ) -> Self {
        Self {
            transparent_field,
            canonical_codec,
        }
    }

    /// Returns whether this value transparently wraps one field.
    #[must_use]
    pub const fn is_transparent(&self) -> bool {
        self.transparent_field.is_some()
    }

    /// Returns the transparent field, if configured.
    #[must_use]
    pub const fn transparent_field(&self) -> Option<&'static FieldMetadata> {
        self.transparent_field
    }

    /// Returns the canonical value codec, if configured.
    #[must_use]
    pub const fn canonical_codec(&self) -> Option<&'static CodecMetadata> {
        self.canonical_codec
    }
}

/// Role-specific metadata payload.
#[derive(Clone, Copy, Debug)]
pub enum RoleMetadata {
    /// Entity-specific payload.
    Entity(EntityMetadata),
    /// Projection-specific payload.
    Projection(ProjectionMetadata),
    /// General model payload.
    Model(ModelMetadata),
    /// Enum-specific payload.
    Enum(EnumMetadata),
    /// Value-specific payload.
    Value(ValueMetadata),
}

impl RoleMetadata {
    /// Returns this payload's role discriminator.
    #[must_use]
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
