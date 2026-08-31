// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Immutable domain metadata anchored to one reflection descriptor root.

#[path = "type_metadata/enum_metadata.rs"]
mod enum_metadata;
#[path = "type_metadata/enum_variant_metadata.rs"]
mod enum_variant_metadata;
#[path = "type_metadata/has_type_metadata.rs"]
mod has_type_metadata;

use std::any::TypeId;

use qubit_reflect::ConcreteGenericDescriptor;
use qubit_reflect::TypeDescriptor;

pub use self::enum_metadata::EnumMetadata;
pub use self::enum_variant_metadata::EnumVariantMetadata;
pub use self::has_type_metadata::HasTypeMetadata;
use crate::EntityMetadata;
use crate::FieldMetadata;
use crate::GenericModelMetadata;
use crate::ModelId;
use crate::ModelMetadata;
use crate::ModelRole;
use crate::ProjectionMetadata;
use crate::PropertyMetadata;
use crate::RoleMetadata;
use crate::ValueMetadata;

static EMPTY_ROLE: RoleMetadata = RoleMetadata::Model(ModelMetadata);

/// Domain semantics for one concrete reflected Rust type.
#[derive(Clone, Copy, Debug)]
pub struct TypeMetadata {
    descriptor: &'static TypeDescriptor,
    model_id: Option<ModelId>,
    fields: &'static [FieldMetadata],
    role: &'static RoleMetadata,
    properties: &'static [PropertyMetadata],
    generic_definition: Option<&'static GenericModelMetadata>,
}

impl TypeMetadata {
    /// Creates generated role-aware metadata over an existing descriptor root.
    #[doc(hidden)]
    #[must_use]
    pub const fn new(
        descriptor: &'static TypeDescriptor,
        model_id: Option<ModelId>,
        fields: &'static [FieldMetadata],
        role: &'static RoleMetadata,
    ) -> Self {
        Self {
            descriptor,
            model_id,
            fields,
            role,
            properties: &[],
            generic_definition: None,
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn with_properties(mut self, properties: &'static [PropertyMetadata]) -> Self {
        self.properties = properties;
        self
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn with_generic_definition(mut self, definition: &'static GenericModelMetadata) -> Self {
        self.generic_definition = Some(definition);
        self
    }

    /// Returns the static metadata generated for `T`.
    #[must_use]
    pub fn of<T: HasTypeMetadata>() -> &'static Self {
        T::type_metadata()
    }

    /// Returns the unique reflection descriptor root.
    #[must_use]
    pub const fn descriptor(&self) -> &'static TypeDescriptor {
        self.descriptor
    }

    /// Returns the concrete Rust type identity.
    #[must_use]
    pub fn type_id(&self) -> TypeId {
        self.descriptor.type_id()
    }

    /// Returns the diagnostic Rust type name.
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        self.descriptor.type_name()
    }

    /// Returns the declared model ID, when registered.
    #[must_use]
    pub const fn model_id(&self) -> Option<ModelId> {
        self.model_id
    }

    /// Returns whether this concrete type declares a model ID.
    #[must_use]
    pub const fn is_registered(&self) -> bool {
        self.model_id.is_some()
    }

    /// Returns structural field overlays in reflection order.
    #[must_use]
    pub const fn fields(&self) -> &'static [FieldMetadata] {
        self.fields
    }

    /// Finds a field by its query name.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&'static FieldMetadata> {
        self.fields.iter().find(|field| field.name() == Some(name))
    }

    /// Finds a field by its source index.
    #[must_use]
    pub fn field_at(&self, index: usize) -> Option<&'static FieldMetadata> {
        self.fields.get(index)
    }

    /// Returns merged field and method properties.
    #[must_use]
    pub fn properties(&self) -> &'static [PropertyMetadata] {
        self.descriptor
            .get_capability(crate::reflect_facade::model_properties_key())
            .map_or(self.properties, |provider| provider())
    }

    /// Finds a property by its public name.
    #[must_use]
    pub fn property(&self, name: &str) -> Option<&'static PropertyMetadata> {
        self.properties().iter().find(|property| property.name() == name)
    }

    /// Returns the generic model template that produced this metadata.
    #[must_use]
    pub const fn generic_definition(&self) -> Option<&'static GenericModelMetadata> {
        self.generic_definition
    }

    /// Returns concrete reflection substitutions for a generic instance.
    #[must_use]
    pub const fn concrete_generic(&self) -> Option<&'static ConcreteGenericDescriptor> {
        self.descriptor.concrete_generic()
    }

    /// Returns the model role tag.
    #[must_use]
    pub const fn role(&self) -> ModelRole {
        self.role.role()
    }

    /// Returns the role-specific metadata payload.
    #[must_use]
    pub const fn role_metadata(&self) -> &'static RoleMetadata {
        self.role
    }

    /// Returns Entity metadata when this is an Entity.
    #[must_use]
    pub const fn as_entity(&self) -> Option<&'static EntityMetadata> {
        match self.role {
            RoleMetadata::Entity(value) => Some(value),
            _ => None,
        }
    }

    /// Returns Projection metadata when this is a Projection.
    #[must_use]
    pub const fn as_projection(&self) -> Option<&'static ProjectionMetadata> {
        match self.role {
            RoleMetadata::Projection(value) => Some(value),
            _ => None,
        }
    }

    /// Returns Model metadata when this is a Model.
    #[must_use]
    pub const fn as_model(&self) -> Option<&'static ModelMetadata> {
        match self.role {
            RoleMetadata::Model(value) => Some(value),
            _ => None,
        }
    }

    /// Returns Enum metadata when this is an Enum.
    #[must_use]
    pub const fn as_enum(&self) -> Option<&'static EnumMetadata> {
        match self.role {
            RoleMetadata::Enum(value) => Some(value),
            _ => None,
        }
    }

    /// Returns Value metadata when this is a Value.
    #[must_use]
    pub const fn as_value(&self) -> Option<&'static ValueMetadata> {
        match self.role {
            RoleMetadata::Value(value) => Some(value),
            _ => None,
        }
    }

    #[doc(hidden)]
    pub(crate) const fn from_descriptor(descriptor: &'static TypeDescriptor) -> Self {
        Self::new(descriptor, None, &[], &EMPTY_ROLE)
    }
}
