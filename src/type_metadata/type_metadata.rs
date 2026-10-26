// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::any::TypeId;

use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::FieldDescriptor;
use qubit_reflect::descriptor::Reflect;

use super::EnumMetadata;
use super::HasTypeMetadata;
#[cfg(feature = "generic")]
use crate::generic::GenericModelMetadata;
use crate::metadata::AbiViolation;
use crate::metadata::EntityMetadata;
use crate::metadata::FieldMetadata;
use crate::metadata::LocalPropertySet;
use crate::metadata::ModelId;
use crate::metadata::ModelMetadata;
use crate::metadata::ModelRole;
use crate::metadata::ProjectionMetadata;
use crate::metadata::PropertyBuildErrors;
use crate::metadata::PropertyFragment;
use crate::metadata::PropertyMetadata;
use crate::metadata::PropertyResolutionError;
use crate::metadata::ResolvedProperties;
use crate::metadata::ResolvedPropertyFragments;
use crate::metadata::RoleMetadata;
use crate::metadata::ValueMetadata;
use crate::reflect_facade::ModelImplResolution;

#[path = "internal/mod.rs"]
mod internal;

use self::internal::abi_violation;
use self::internal::descriptor_for;
use self::internal::validate_fields;
use self::internal::validate_properties;
use self::internal::validate_role;

/// Domain semantics for one concrete reflected Rust type.
///
/// This immutable overlay borrows process-lifetime descriptors and
/// declarations. Obtain generated metadata with [`Self::of`]; a stable model ID
/// is optional and is distinct from the concrete Rust [`TypeId`]. Inspecting
/// declarations does not execute their validation rules or inspect an instance.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// #[Model(id = "qubit_model_metadata.TypeMetadataNote")]
/// struct Note { #[text(non_blank)] title: String }
/// # fn main() {
/// let metadata = TypeMetadata::of::<Note>();
/// assert_eq!(
///     metadata.model_id().expect("named model").as_str(),
///     "qubit_model_metadata.TypeMetadataNote"
/// );
/// let title = metadata.field("title").expect("declared field");
/// assert!(title.text_constraint().expect("text declaration").is_non_blank());
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct TypeMetadata {
    /// The process-local identity of the reflected Rust type.
    type_id: TypeId,
    /// The diagnostic name of the reflected Rust type.
    type_name: &'static str,
    /// Rebuilds a descriptor in the caller's reflection context.
    descriptor_factory: for<'ctx> fn(&'ctx ReflectionContext) -> TypeDescriptor<'ctx>,
    /// The stable model ID, or `None` for unregistered types.
    model_id: Option<ModelId>,
    /// Field overlays in reflection declaration order.
    fields: &'static [FieldMetadata],
    /// Role-specific metadata for the reflected type.
    role: &'static RoleMetadata,
    /// Merged field and method properties for the reflected type.
    properties: LocalPropertySet,
    /// Unmerged field property declarations for types without `ModelImpl`.
    property_fragments: &'static [PropertyFragment],
    /// The generic definition that produced this concrete instance, if any.
    #[cfg(feature = "generic")]
    generic_definition: Option<&'static GenericModelMetadata>,
    /// Lazily resolved metadata providers for concrete model types nested in
    /// this type's declared field expressions.
    nested_metadata_providers: &'static [fn() -> Option<&'static TypeMetadata>],
}

impl TypeMetadata {
    /// Returns the static metadata generated for `T`.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The reflected Rust type whose generated metadata is returned.
    ///
    /// # Returns
    ///
    /// The process-lifetime metadata generated for `T`.
    ///
    /// # Panics
    ///
    /// Panics when generated metadata fails ABI validation for `T`.
    #[must_use]
    pub fn of<T: HasTypeMetadata>() -> &'static Self {
        Self::try_of::<T>().unwrap_or_else(|error| panic!("{error}"))
    }

    /// Returns the static metadata generated for `T` after ABI validation.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The reflected Rust type whose generated metadata is validated.
    ///
    /// # Returns
    ///
    /// `Ok` with the process-lifetime metadata when its descriptor matches `T`.
    ///
    /// # Errors
    ///
    /// Returns a structured violation when generated metadata disagrees with
    /// the unique reflection descriptor for `T`.
    #[must_use = "handle generated metadata ABI violations"]
    pub fn try_of<T: HasTypeMetadata>() -> Result<&'static Self, AbiViolation> {
        let metadata = <T as crate::__private::TypeMetadataProvider>::__type_metadata();
        metadata.validate_for::<T>()?;
        Ok(metadata)
    }

    /// Creates generated role-aware metadata over an existing descriptor root.
    #[doc(hidden)]
    #[must_use]
    #[inline]
    pub(crate) fn new<T: Reflect + 'static>(
        model_id: Option<ModelId>,
        fields: &'static [FieldMetadata],
        role: &'static RoleMetadata,
    ) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: core::any::type_name::<T>(),
            descriptor_factory: descriptor_for::<T>,
            model_id,
            fields,
            role,
            properties: LocalPropertySet::new(&[]),
            property_fragments: &[],
            #[cfg(feature = "generic")]
            generic_definition: None,
            nested_metadata_providers: &[],
        }
    }

    /// Adds generated property metadata to this immutable overlay.
    #[doc(hidden)]
    #[must_use]
    #[inline]
    pub(crate) const fn with_properties(mut self, properties: &'static [PropertyMetadata]) -> Self {
        self.properties = LocalPropertySet::new(properties);
        self
    }

    /// Adds generated field property fragments to this immutable overlay.
    #[doc(hidden)]
    #[must_use]
    #[inline]
    pub(crate) const fn with_property_fragments(mut self, fragments: &'static [PropertyFragment]) -> Self {
        self.property_fragments = fragments;
        self
    }

    /// Adds lazy metadata providers for concrete nested field types.
    #[doc(hidden)]
    #[must_use]
    #[inline]
    pub(crate) const fn with_nested_metadata_providers(
        mut self,
        providers: &'static [fn() -> Option<&'static TypeMetadata>],
    ) -> Self {
        self.nested_metadata_providers = providers;
        self
    }

    /// Resolves retained metadata for concrete types nested in field
    /// declarations.
    ///
    /// The iterator yields the process-lifetime [`TypeMetadata`] values
    /// provided by this type's nested metadata providers. It borrows `self`
    /// for the lifetime of the iterator; each yielded metadata value is
    /// independently `'static`.
    pub(crate) fn nested_metadata(&self) -> impl Iterator<Item = &'static TypeMetadata> + '_ {
        self.nested_metadata_providers.iter().filter_map(|provider| provider())
    }

    /// Records the generic definition that produced this concrete instance.
    #[doc(hidden)]
    #[must_use]
    #[inline]
    #[cfg(feature = "generic")]
    pub(crate) const fn with_generic_definition(mut self, definition: &'static GenericModelMetadata) -> Self {
        self.generic_definition = Some(definition);
        self
    }

    /// Returns this type's descriptor in the supplied reflection context.
    ///
    /// # Returns
    ///
    /// A descriptor tied to `context`; it cannot outlive that context.
    #[must_use]
    #[inline]
    pub fn descriptor_in<'ctx>(&self, context: &'ctx ReflectionContext) -> TypeDescriptor<'ctx> {
        (self.descriptor_factory)(context)
    }

    /// Resolves one retained field identity against this metadata's descriptor.
    pub(crate) fn field_descriptor_in<'ctx>(
        &self,
        field: &FieldMetadata,
        context: &'ctx ReflectionContext,
    ) -> Option<FieldDescriptor<'ctx>> {
        let location = field.location()?;
        if location.owner() != self.type_id {
            return None;
        }
        let descriptor = self.descriptor_in(context);
        if let Some(variant_index) = location.variant() {
            descriptor
                .variants()
                .iter()
                .find(|variant| variant.index() == variant_index)?
                .fields()
                .iter()
                .find(|candidate| candidate.index() == location.index())
        } else {
            descriptor
                .fields()
                .iter()
                .find(|candidate| candidate.index() == location.index())
        }
    }

    /// Returns the concrete Rust type identity.
    ///
    /// # Returns
    ///
    /// The `TypeId` of the reflected concrete Rust type.
    #[must_use]
    #[inline]
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Returns the diagnostic Rust type name.
    ///
    /// # Returns
    ///
    /// The static diagnostic name reported by the reflection descriptor.
    #[must_use]
    #[inline]
    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// Returns the declared model ID, when registered.
    ///
    /// # Returns
    ///
    /// `Some` when the concrete type declares an ID, or `None` otherwise.
    #[inline]
    pub const fn model_id(&self) -> Option<ModelId> {
        self.model_id
    }

    /// Returns whether this concrete type declares a model ID.
    ///
    /// # Returns
    ///
    /// `true` when a model ID is declared, and `false` otherwise.
    #[must_use]
    #[inline]
    pub const fn is_registered(&self) -> bool {
        self.model_id.is_some()
    }

    /// Returns structural field overlays in reflection order.
    ///
    /// # Returns
    ///
    /// The process-lifetime field overlays in source order.
    #[must_use]
    #[inline]
    pub const fn fields(&self) -> &'static [FieldMetadata] {
        self.fields
    }

    /// Finds a field by its query name.
    ///
    /// # Parameters
    ///
    /// - `name`: The exact field query name to find.
    ///
    /// # Returns
    ///
    /// `Some` with the matching field, or `None` when no field has that name.
    pub fn field(&self, name: &str) -> Option<&'static FieldMetadata> {
        self.fields.iter().find(|field| field.name() == Some(name))
    }

    /// Finds a field by its source index.
    ///
    /// # Parameters
    ///
    /// - `index`: The zero-based source-order field index.
    ///
    /// # Returns
    ///
    /// `Some` with the field at that index, or `None` when it is out of range.
    #[inline]
    pub fn field_at(&self, index: usize) -> Option<&'static FieldMetadata> {
        self.fields.get(index)
    }

    /// Returns effective field/getter/setter declarations from the global
    /// registry.
    ///
    /// # Returns
    ///
    /// The effective declaration fragments, borrowing static metadata or
    /// owning dynamically merged fragments.
    ///
    /// # Errors
    ///
    /// Returns reflection initialization or intrinsic capability conflicts.
    pub fn property_fragments(&'static self) -> Result<ResolvedPropertyFragments, PropertyResolutionError> {
        self.property_fragments_in(ReflectionContext::global())
    }

    /// Returns declarations from `registry`, falling back to local fields when
    /// no implementation overlay is present. Never accesses the global
    /// registry. Dynamic fragments are owned by the returned view.
    ///
    /// # Parameters
    ///
    /// - `registry`: The initialized reflection snapshot used for resolution.
    ///
    /// # Returns
    ///
    /// The effective declaration fragments, borrowing static metadata or
    /// owning dynamically merged fragments.
    ///
    /// # Errors
    ///
    /// Returns intrinsic capability conflicts for an invalid owner.
    pub fn property_fragments_in(
        &'static self,
        registry: &ReflectionContext,
    ) -> Result<ResolvedPropertyFragments, PropertyResolutionError> {
        Ok(match crate::reflect_facade::model_impl_metadata(self, registry)? {
            None => ResolvedPropertyFragments::Static(self.property_fragments),
            Some(ModelImplResolution::Merged(metadata)) => metadata.fragments(),
        })
    }

    /// Returns effective properties from the process-wide reflection snapshot.
    /// Dynamic properties are owned by the returned view.
    ///
    /// # Returns
    ///
    /// The effective properties, borrowing static metadata or owning dynamic
    /// results.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyResolutionError::Reflection`] for initialization
    /// failures, [`PropertyResolutionError::Capability`] for intrinsic
    /// conflicts, and [`PropertyResolutionError::Assembly`] for
    /// incompatible declarations.
    pub fn try_properties(&'static self) -> Result<ResolvedProperties, PropertyResolutionError> {
        self.try_properties_in(ReflectionContext::global())
    }

    /// Returns effective properties in `registry`. Never initializes global
    /// state. Dynamic results are owned by the returned view and are not cached
    /// by this method; use [`crate::registry::ModelRegistry::properties_for`]
    /// to reuse merges within a registry's lifetime.
    ///
    /// # Parameters
    ///
    /// - `registry`: The initialized reflection snapshot used for resolution.
    ///
    /// # Returns
    ///
    /// The effective properties, borrowing static metadata or owning dynamic
    /// results.
    ///
    /// # Errors
    ///
    /// Returns intrinsic capability conflicts or incompatible property
    /// assembly.
    pub fn try_properties_in(
        &'static self,
        registry: &ReflectionContext,
    ) -> Result<ResolvedProperties, PropertyResolutionError> {
        match crate::reflect_facade::model_impl_metadata(self, registry)? {
            None => Ok(ResolvedProperties::Static(&self.properties)),
            Some(ModelImplResolution::Merged(metadata)) => {
                metadata.try_properties().map_err(PropertyResolutionError::Assembly)
            }
        }
    }

    /// Returns declaration-local field properties without consulting any
    /// registry.
    #[must_use]
    #[inline]
    pub(crate) const fn local_properties(&'static self) -> &'static LocalPropertySet {
        &self.properties
    }

    /// Finds an effective property in the global snapshot.
    ///
    /// Returns `None` for an absent name and propagates initialization or
    /// capability or assembly errors.
    ///
    /// # Parameters
    ///
    /// - `name`: The exact effective property name to find.
    ///
    /// # Returns
    ///
    /// `Ok(Some(property))` when found, `Ok(None)` when absent, or an error
    /// when reflection initialization, capability resolution, or assembly
    /// fails.
    ///
    /// # Errors
    ///
    /// Returns reflection initialization, capability, or property assembly
    /// errors.
    pub fn try_property(&'static self, name: &str) -> Result<Option<PropertyMetadata>, PropertyResolutionError> {
        self.try_properties()
            .map(|properties| properties.property(name).copied())
    }

    /// Finds an effective property in `registry` without accessing global
    /// state.
    ///
    /// # Parameters
    ///
    /// - `registry`: The initialized reflection snapshot used for resolution.
    /// - `name`: The exact effective property name to find.
    ///
    /// # Returns
    ///
    /// `Ok(Some(property))` when found, `Ok(None)` when absent, or an error
    /// when capability resolution or assembly fails.
    ///
    /// # Errors
    ///
    /// Returns capability or property assembly errors.
    pub fn try_property_in(
        &'static self,
        registry: &ReflectionContext,
        name: &str,
    ) -> Result<Option<PropertyMetadata>, PropertyResolutionError> {
        self.try_properties_in(registry)
            .map(|properties| properties.property(name).copied())
    }

    /// Returns the generic model template that produced this metadata.
    ///
    /// # Returns
    ///
    /// `Some` with the generic definition for a concrete generic model, or
    /// `None` when this metadata has no generic definition.
    #[inline]
    #[cfg(feature = "generic")]
    pub const fn generic_definition(&self) -> Option<&'static GenericModelMetadata> {
        self.generic_definition
    }

    /// Returns the model role tag.
    ///
    /// # Returns
    ///
    /// The role declared for this reflected type.
    #[must_use]
    #[inline]
    pub const fn role(&self) -> ModelRole {
        self.role.role()
    }

    /// Returns the role-specific metadata payload.
    ///
    /// # Returns
    ///
    /// The process-lifetime metadata for this type's role.
    #[must_use]
    #[inline]
    pub const fn role_metadata(&self) -> &'static RoleMetadata {
        self.role
    }

    /// Returns Entity metadata when this is an Entity.
    ///
    /// # Returns
    ///
    /// `Some` with entity metadata for an Entity, or `None` for other roles.
    #[inline]
    pub const fn as_entity(&self) -> Option<&'static EntityMetadata> {
        match self.role {
            RoleMetadata::Entity(value) => Some(value),
            _ => None,
        }
    }

    /// Returns Projection metadata when this is a Projection.
    ///
    /// # Returns
    ///
    /// `Some` with projection metadata for a Projection, or `None` for other
    /// roles.
    #[inline]
    pub const fn as_projection(&self) -> Option<&'static ProjectionMetadata> {
        match self.role {
            RoleMetadata::Projection(value) => Some(value),
            _ => None,
        }
    }

    /// Returns Model metadata when this is a Model.
    ///
    /// # Returns
    ///
    /// `Some` with model metadata for a Model, or `None` for other roles.
    #[inline]
    pub const fn as_model(&self) -> Option<&'static ModelMetadata> {
        match self.role {
            RoleMetadata::Model(value) => Some(value),
            _ => None,
        }
    }

    /// Returns Enum metadata when this is an Enum.
    ///
    /// # Returns
    ///
    /// `Some` with enum metadata for an Enum, or `None` for other roles.
    #[inline]
    pub const fn as_enum(&self) -> Option<&'static EnumMetadata> {
        match self.role {
            RoleMetadata::Enum(value) => Some(value),
            _ => None,
        }
    }

    /// Returns Value metadata when this is a Value.
    ///
    /// # Returns
    ///
    /// `Some` with value metadata for a Value, or `None` for other roles.
    #[inline]
    pub const fn as_value(&self) -> Option<&'static ValueMetadata> {
        match self.role {
            RoleMetadata::Value(value) => Some(value),
            _ => None,
        }
    }

    /// Verifies that generated metadata is anchored to `T` and internally
    /// consistent before it crosses the public ABI boundary.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The Rust type to which this metadata must be anchored.
    ///
    /// # Panics
    ///
    /// Panics when the descriptor or generated metadata violates the ABI.
    #[doc(hidden)]
    pub fn assert_valid_for<T: 'static>(&self) {
        self.validate_for::<T>().unwrap_or_else(|error| panic!("{error}"));
    }

    /// Checks that generated metadata is anchored to `T`.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The Rust type to which this metadata must be anchored.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the metadata is valid for `T`.
    ///
    /// # Errors
    ///
    /// Returns an ABI violation when the descriptor or generated metadata does
    /// not match `T` or fails internal consistency checks.
    #[doc(hidden)]
    #[must_use = "handle generated metadata ABI violations"]
    pub fn validate_for<T: 'static>(&self) -> Result<(), AbiViolation> {
        if self.type_id != TypeId::of::<T>() {
            return Err(abi_violation(
                "QMM-ABI-001",
                "metadata descriptor does not describe the requested Rust type",
            ));
        }
        Ok(())
    }

    /// Checks that this metadata remains attached to `descriptor`.
    pub(crate) fn validate_descriptor(&self, descriptor: &TypeDescriptor) -> Result<(), AbiViolation> {
        if self.type_id != descriptor.type_id() {
            return Err(abi_violation(
                "QMM-ABI-002",
                "metadata is attached to a different reflection root",
            ));
        }

        if !matches!(self.role, RoleMetadata::Enum(_)) {
            validate_fields(self.fields, descriptor.fields(), *descriptor, "QMM-ABI-003")?;
        }
        #[cfg(feature = "generic")]
        {
            if self.model_id.is_some() && self.generic_definition.is_some() {
                return Err(abi_violation(
                    "QMM-ABI-022",
                    "concrete metadata cannot own both a model ID and a generic definition",
                ));
            }
            if let Some(definition) = self.generic_definition
                && (descriptor
                    .find_definition()
                    .is_none_or(|actual| actual.path() != definition.definition_path())
                    || definition.role() != self.role())
            {
                return Err(abi_violation(
                    "QMM-ABI-022",
                    "generic metadata does not match its concrete instance",
                ));
            }
        }
        if validate_properties(self.properties.properties(), self.fields, *descriptor).is_err() {
            return Err(abi_violation(
                "QMM-ABI-004",
                "generated field properties are internally inconsistent",
            ));
        }
        validate_role(self, *descriptor)
    }

    /// Verifies a property overlay supplied by a generated capability.
    ///
    /// # Parameters
    ///
    /// - `properties`: The generated properties to validate against this type.
    ///
    /// # Returns
    ///
    /// `Ok(())` when every property agrees with the fields and descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyBuildErrors`] when the supplied properties contain
    /// duplicate names, missing sources, foreign fields, or incompatible
    /// field, getter, or setter types.
    #[doc(hidden)]
    #[must_use = "handle invalid generated property metadata"]
    pub fn validate_properties_in(
        &self,
        context: &ReflectionContext,
        properties: &[PropertyMetadata],
    ) -> Result<(), PropertyBuildErrors> {
        validate_properties(properties, self.fields, self.descriptor_in(context))
    }
}
