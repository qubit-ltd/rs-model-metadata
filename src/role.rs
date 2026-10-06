// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! The five supported model roles and their role-specific payloads.

use crate::metadata::CodecMetadata;
use crate::metadata::DeclaredEntityTarget;
use crate::metadata::EnumMetadata;
use crate::metadata::FieldMetadata;

/// The semantic role assigned by a model macro.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::ModelRole;
///
/// let role = ModelRole::Entity;
/// assert_eq!(role, ModelRole::Entity);
/// ```
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
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::EntityMetadata;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// # mod example {
/// use qubit_id::Id;
/// use qubit_model_derive::Entity;
///
/// #[Entity(id = "example.Customer")]
/// pub struct Customer {
///     #[identifier]
///     pub id: Id,
/// }
/// # }
///
/// let entity: &EntityMetadata = TypeMetadata::of::<example::Customer>().as_entity().expect("entity metadata");
/// assert_eq!(entity.identifier().name(), Some("id"));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct EntityMetadata {
    /// The field that supplies the entity identifier.
    identifier: &'static FieldMetadata,
}

impl EntityMetadata {
    /// Creates entity metadata.
    ///
    /// # Parameters
    ///
    /// - `identifier`: the reflected field that supplies the entity identity.
    ///
    /// # Returns
    ///
    /// Entity metadata retaining the process-lifetime identifier field.
    #[must_use]
    #[inline]
    pub(crate) const fn new(identifier: &'static FieldMetadata) -> Self {
        Self { identifier }
    }

    /// Returns the entity identifier field.
    ///
    /// # Returns
    ///
    /// The process-lifetime field metadata used as this entity's identifier.
    #[must_use]
    #[inline]
    pub const fn identifier(&self) -> &'static FieldMetadata {
        self.identifier
    }
}

/// Projection-specific metadata.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::ProjectionMetadata;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// # mod example {
/// use qubit_id::Id;
/// use qubit_model_derive::Entity;
/// use qubit_model_derive::Projection;
///
/// #[Entity(id = "example.Source")]
/// pub struct Source {
///     #[identifier]
///     pub id: Id,
/// }
///
/// #[Projection(id = "example.View", source = Source)]
/// pub struct View {
///     #[identifier]
///     pub id: Id,
/// }
/// # }
///
/// let projection: &ProjectionMetadata = TypeMetadata::of::<example::View>().as_projection().expect("projection metadata");
/// assert!(projection.is_fixed());
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ProjectionMetadata {
    /// The field that supplies the projection identifier.
    identifier: &'static FieldMetadata,
    /// The optional entity target from which projection fields originate.
    source: Option<&'static DeclaredEntityTarget>,
}

impl ProjectionMetadata {
    /// Creates projection metadata.
    ///
    /// # Parameters
    ///
    /// - `identifier`: the reflected field that supplies the projection
    ///   identity.
    /// - `source`: the optional declared entity source for the projection.
    ///
    /// # Returns
    ///
    /// Projection metadata retaining its identifier and source declaration.
    #[must_use]
    #[inline]
    pub(crate) const fn new(
        identifier: &'static FieldMetadata,
        source: Option<&'static DeclaredEntityTarget>,
    ) -> Self {
        Self { identifier, source }
    }

    /// Returns the projection identifier field.
    ///
    /// # Returns
    ///
    /// The process-lifetime field metadata used as this projection's
    /// identifier.
    #[must_use]
    #[inline]
    pub const fn identifier(&self) -> &'static FieldMetadata {
        self.identifier
    }

    /// Returns the optional declared source without consulting a registry.
    ///
    /// # Returns
    ///
    /// The declared entity target when one was configured, or `None` for an
    /// open projection.
    #[must_use]
    #[inline]
    pub const fn source(&self) -> Option<&'static DeclaredEntityTarget> {
        self.source
    }

    /// Returns whether undeclared source fields are accepted.
    ///
    /// # Returns
    ///
    /// `true` when this projection has no declared source.
    #[must_use]
    #[inline]
    pub const fn is_open(&self) -> bool {
        self.source.is_none()
    }

    /// Returns whether the projection field set is fixed.
    ///
    /// # Returns
    ///
    /// `true` when this projection declares a source entity.
    #[must_use]
    #[inline]
    pub const fn is_fixed(&self) -> bool {
        self.source.is_some()
    }
}

/// Model-specific metadata, intentionally empty in the first version.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::ModelMetadata;
/// use qubit_model_metadata::metadata::ModelRole;
/// use qubit_model_metadata::metadata::RoleMetadata;
///
/// let model = ModelMetadata;
/// assert_eq!(RoleMetadata::Model(model).role(), ModelRole::Model);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ModelMetadata;

/// Value-specific metadata.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::RoleMetadata;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::metadata::ValueMetadata;
///
/// # mod example {
/// use qubit_model_derive::Value;
///
/// #[Value(transparent)]
/// pub struct UserId(pub u64);
/// # }
///
/// let RoleMetadata::Value(value) = TypeMetadata::of::<example::UserId>().role_metadata() else {
///     panic!("derived value metadata");
/// };
/// let value: &ValueMetadata = value;
/// assert!(value.is_transparent());
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ValueMetadata {
    /// The sole wrapped field when the value is transparent.
    transparent_field: Option<&'static FieldMetadata>,
    /// The codec that represents the complete value, when configured.
    canonical_codec: Option<&'static CodecMetadata>,
}

impl ValueMetadata {
    /// Creates value metadata.
    ///
    /// # Parameters
    ///
    /// - `transparent_field`: the sole wrapped field when the value is
    ///   transparent.
    /// - `canonical_codec`: the codec that represents the complete value, if
    ///   configured.
    ///
    /// # Returns
    ///
    /// Value metadata retaining the configured transparent-field and codec
    /// semantics.
    #[must_use]
    #[inline]
    pub(crate) const fn new(
        transparent_field: Option<&'static FieldMetadata>,
        canonical_codec: Option<&'static CodecMetadata>,
    ) -> Self {
        Self {
            transparent_field,
            canonical_codec,
        }
    }

    /// Returns whether this value transparently wraps one field.
    ///
    /// # Returns
    ///
    /// `true` when a transparent field is configured.
    #[must_use]
    #[inline]
    pub const fn is_transparent(&self) -> bool {
        self.transparent_field.is_some()
    }

    /// Returns the transparent field, if configured.
    ///
    /// # Returns
    ///
    /// The sole wrapped field for a transparent value, or `None` otherwise.
    #[must_use]
    #[inline]
    pub const fn transparent_field(&self) -> Option<&'static FieldMetadata> {
        self.transparent_field
    }

    /// Returns the canonical value codec, if configured.
    ///
    /// # Returns
    ///
    /// The codec representing the whole value, or `None` when no canonical
    /// codec was configured.
    #[must_use]
    #[inline]
    pub const fn canonical_codec(&self) -> Option<&'static CodecMetadata> {
        self.canonical_codec
    }
}

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
