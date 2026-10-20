// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable domain metadata anchored to one reflection descriptor root.

#[path = "type_metadata/enum_metadata.rs"]
mod enum_metadata;
#[path = "type_metadata/enum_variant_metadata.rs"]
mod enum_variant_metadata;
#[path = "type_metadata/has_type_metadata.rs"]
mod has_type_metadata;

use std::any::TypeId;
use std::collections::HashSet;

use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::FieldDescriptor;
use qubit_reflect::descriptor::Fields;
use qubit_reflect::descriptor::Reflect;
use qubit_reflect::descriptor::TypeKind;

pub use self::enum_metadata::EnumMetadata;
pub use self::enum_variant_metadata::EnumVariantMetadata;
pub use self::has_type_metadata::HasTypeMetadata;
#[cfg(feature = "generic")]
use crate::generic::GenericModelMetadata;
use crate::metadata::AbiViolation;
use crate::metadata::ConstraintMetadata;
use crate::metadata::EntityMetadata;
use crate::metadata::FieldAttributeMetadata;
use crate::metadata::FieldMetadata;
use crate::metadata::GetterMetadata;
use crate::metadata::GetterOutputKind;
use crate::metadata::LocalPropertySet;
use crate::metadata::ModelId;
use crate::metadata::ModelMetadata;
use crate::metadata::ModelRole;
use crate::metadata::ProjectionMetadata;
use crate::metadata::PropertyBuildError;
use crate::metadata::PropertyBuildErrorKind;
use crate::metadata::PropertyBuildErrors;
use crate::metadata::PropertyFragment;
use crate::metadata::PropertyMetadata;
use crate::metadata::PropertyResolutionError;
use crate::metadata::ResolvedProperties;
use crate::metadata::ResolvedPropertyFragments;
use crate::metadata::RoleMetadata;
use crate::metadata::SelectorPosition;
use crate::metadata::ValueMetadata;
use crate::reflect_facade::ModelImplResolution;
use crate::transparent_descriptor::transparent_descriptor;

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

    /// Resolves metadata providers retained for nested concrete field types.
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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

fn descriptor_for<'ctx, T: Reflect + 'static>(context: &'ctx ReflectionContext) -> TypeDescriptor<'ctx> {
    TypeDescriptor::of_in::<T>(context)
}

/// Verifies that field overlays mirror the reflection descriptor exactly.
fn validate_fields(
    metadata: &[FieldMetadata],
    reflected: Fields<'_>,
    descriptor: TypeDescriptor<'_>,
    code: &'static str,
) -> Result<(), AbiViolation> {
    let reflected_count = metadata.iter().filter(|field| !field.is_opaque()).count();
    if reflected_count != reflected.len() {
        return Err(abi_violation(
            code,
            "field metadata count differs from reflection metadata",
        ));
    }
    for (index, field) in metadata.iter().enumerate() {
        let Some(location) = field.location() else {
            return Err(abi_violation(
                code,
                "concrete field metadata has no declaration identity",
            ));
        };
        if location.owner() != descriptor.type_id() || location.index() != index || field.index() != index {
            return Err(abi_violation(code, "field metadata identity differs from source order"));
        }
        if field.is_opaque() {
            validate_field_semantics(field, descriptor.context())?;
            continue;
        }
        let Some(reflect) = reflected.iter().find(|candidate| candidate.index() == field.index()) else {
            return Err(abi_violation(
                code,
                "visible field metadata has no reflection field at its source index",
            ));
        };
        if location.variant() != reflect.declaring_variant().map(|variant| variant.index())
            || !core::ptr::eq(reflect.context(), descriptor.context())
            || reflect.declaring_type().type_id() != descriptor.type_id()
        {
            return Err(abi_violation(
                code,
                "field metadata identity differs from reflection source order",
            ));
        }
        validate_field_semantics(field, descriptor.context())?;
    }
    let mut key_parts = metadata
        .iter()
        .filter_map(|field| field.key_part().map(|key_part| key_part.order()))
        .collect::<Vec<_>>();
    key_parts.sort_unstable();
    if key_parts.iter().copied().ne(0..key_parts.len()) {
        return Err(abi_violation(
            "QMM-ABI-019",
            "key-part orders must be unique and contiguous",
        ));
    }
    Ok(())
}

/// Verifies mutually compatible field-level semantic declarations.
fn validate_field_semantics(field: &FieldMetadata, context: &ReflectionContext) -> Result<(), AbiViolation> {
    // Every counter is bounded by the slice length, so usize cannot overflow
    // even for malformed generated input containing more than 255 repeats.
    let mut singleton_counts = [0_usize; 9];
    let mut constraints = Vec::new();
    let mut validators = Vec::new();
    for attribute in field.attributes() {
        match attribute {
            FieldAttributeMetadata::Identifier(_) => singleton_counts[0] += 1,
            FieldAttributeMetadata::Unique(_) => singleton_counts[1] += 1,
            FieldAttributeMetadata::Reference(_) => singleton_counts[2] += 1,
            FieldAttributeMetadata::KeyPart(_) => singleton_counts[3] += 1,
            FieldAttributeMetadata::Codec(_) => singleton_counts[4] += 1,
            FieldAttributeMetadata::Redact(_) => singleton_counts[5] += 1,
            FieldAttributeMetadata::Serde(value) => {
                singleton_counts[6] += 1;
                if !core::ptr::eq(*value, field.serde()) {
                    return Err(abi_violation(
                        "QMM-ABI-017",
                        "serde occurrence does not match the cached field value",
                    ));
                }
            }
            FieldAttributeMetadata::Opaque => singleton_counts[7] += 1,
            FieldAttributeMetadata::Indexed(_) => singleton_counts[8] += 1,
            FieldAttributeMetadata::Constraint(value) => constraints.push(*value),
            FieldAttributeMetadata::Validator(value) => validators.push(*value),
        }
    }
    if singleton_counts.into_iter().any(|count| count > 1) {
        return Err(abi_violation(
            "QMM-ABI-015",
            "a field contains duplicate singleton semantics",
        ));
    }
    if constraints.len() != field.constraints().len()
        || !constraints
            .iter()
            .zip(field.constraints())
            .all(|(left, right)| core::ptr::eq(*left, right))
        || validators.len() != field.validators().len()
        || !validators
            .iter()
            .zip(field.validators())
            .all(|(left, right)| core::ptr::eq(*left, right))
    {
        return Err(abi_violation(
            "QMM-ABI-016",
            "field occurrence lists and cached slices disagree",
        ));
    }
    let concrete_type = field
        .reflect_in(context)
        .and_then(|reflected| reflected.find_concrete_type());
    validate_constraint_kinds(field.constraints(), concrete_type, true)?;
    validate_validators(field.validators())?;
    validate_codec(
        field.codec(),
        concrete_type.map(|descriptor| descriptor.type_id()),
        crate::metadata::CodecSource::Field,
    )?;
    Ok(())
}

/// Verifies that validator declarations have non-empty, unique dependencies.
fn validate_validators(validators: &[crate::metadata::ValidatorMetadata]) -> Result<(), AbiViolation> {
    for validator in validators {
        let mut parameter_names = HashSet::with_capacity(validator.params().len());
        if validator
            .params()
            .iter()
            .any(|argument| !parameter_names.insert(argument.name()))
        {
            return Err(abi_violation("QMM-ABI-018", "validator parameter names must be unique"));
        }
        for (index, dependency) in validator.depends_on().iter().enumerate() {
            if dependency.is_empty() || validator.depends_on()[..index].contains(dependency) {
                return Err(abi_violation(
                    "QMM-ABI-018",
                    "validator dependency paths must be non-empty and unique",
                ));
            }
        }
    }
    Ok(())
}

/// Verifies constraint kinds against the reflected field type.
fn validate_constraint_kinds(
    constraints: &[ConstraintMetadata],
    descriptor: Option<TypeDescriptor<'_>>,
    allow_selectors: bool,
) -> Result<(), AbiViolation> {
    let mut kinds = HashSet::with_capacity(constraints.len());
    for constraint in constraints {
        let kind = match constraint {
            ConstraintMetadata::Text(_) => 0,
            ConstraintMetadata::Decimal(_) => 1,
            ConstraintMetadata::Time(_) => 2,
            ConstraintMetadata::Sequence(sequence) => {
                if let Some(selector) = sequence.element() {
                    if !allow_selectors {
                        return Err(abi_violation("QMM-ABI-020", "selector semantics must be non-recursive"));
                    }
                    validate_selector(selector, SelectorPosition::Element, descriptor)?;
                }
                3
            }
            ConstraintMetadata::Map(map) => {
                for (selector, position) in [
                    (map.key(), SelectorPosition::MapKey),
                    (map.value(), SelectorPosition::MapValue),
                ] {
                    if let Some(selector) = selector {
                        if !allow_selectors {
                            return Err(abi_violation("QMM-ABI-020", "selector semantics must be non-recursive"));
                        }
                        validate_selector(selector, position, descriptor)?;
                    }
                }
                4
            }
        };
        if !kinds.insert(kind) {
            return Err(abi_violation(
                "QMM-ABI-021",
                "a field contains duplicate constraint kinds",
            ));
        }
    }
    Ok(())
}

/// Verifies one nested selector against its structural position and type.
fn validate_selector(
    selector: &crate::metadata::SelectorMetadata,
    position: SelectorPosition,
    descriptor: Option<TypeDescriptor<'_>>,
) -> Result<(), AbiViolation> {
    if selector.position() != position {
        return Err(abi_violation(
            "QMM-ABI-020",
            "selector has the wrong structural position",
        ));
    }
    let Some(selected_type) = descriptor
        .and_then(transparent_descriptor)
        .and_then(|descriptor| selector_type_descriptor(descriptor, position))
    else {
        return Err(abi_violation(
            "QMM-ABI-020",
            "selector position is incompatible with the field type",
        ));
    };
    validate_constraint_kinds(selector.constraints(), Some(selected_type), false)?;
    validate_validators(selector.validators())?;
    validate_codec(
        selector.codec(),
        Some(selected_type.type_id()),
        crate::metadata::CodecSource::Selector(position),
    )
}

/// Verifies that a codec declaration is valid for its expected type and source.
fn validate_codec(
    codec: Option<&crate::metadata::CodecMetadata>,
    expected_type: Option<TypeId>,
    source: crate::metadata::CodecSource,
) -> Result<(), AbiViolation> {
    let Some(codec) = codec else {
        return Ok(());
    };
    if codec.source() != source {
        return Err(abi_violation(
            "QMM-ABI-025",
            "codec source differs from its metadata position",
        ));
    }
    let _ = expected_type;
    Ok(())
}

/// Returns the nested type selected by a collection position.
fn selector_type_descriptor(descriptor: TypeDescriptor<'_>, position: SelectorPosition) -> Option<TypeDescriptor<'_>> {
    let arguments = descriptor.generic_arguments();
    let index = match (descriptor.kind(), position) {
        (
            TypeKind::Sequence(_) | TypeKind::Set(_) | TypeKind::Array(_) | TypeKind::Slice(_),
            SelectorPosition::Element,
        ) => 0,
        (TypeKind::Map(_), SelectorPosition::MapKey) => 0,
        (TypeKind::Map(_), SelectorPosition::MapValue) => 1,
        _ => return None,
    };
    arguments.get_at(index)?.type_argument().ok()?.find_concrete_type()
}

/// Resolves a type expression's concrete type ID when available.
fn property_type_descriptor<'ctx>(
    property: &PropertyMetadata,
    fields: &[FieldMetadata],
    context: &'ctx ReflectionContext,
) -> Option<TypeDescriptor<'ctx>> {
    property
        .field()
        .or_else(|| fields.iter().find(|field| field.name() == Some(property.name())))?
        .reflect_in(context)?
        .find_concrete_type()
}

/// Checks whether an explicit property expression agrees with an accessor
/// signature when no concrete backing field can establish the type identity.
fn property_accessor_expression_matches(
    property: &PropertyMetadata,
    accessor: qubit_reflect::descriptor::TypeExpression<'_>,
    context: &ReflectionContext,
) -> bool {
    let Some(property_expression) = property.type_expression_in(context) else {
        return true;
    };
    property_expression.display().to_string() == accessor.display().to_string()
}

/// Checks whether a getter's concrete output agrees with the property type.
fn getter_type_compatible(
    property_type: Option<TypeDescriptor<'_>>,
    getter: &GetterMetadata,
    context: &ReflectionContext,
) -> bool {
    let Some(property_type) = property_type else {
        return true;
    };
    let output_type_id = getter.output_type_id();
    if property_type.type_id() == output_type_id {
        return true;
    }
    if text_accessor_type_compatible(property_type, output_type_id, context) {
        return true;
    }
    if getter.output_kind() == GetterOutputKind::BorrowedSlice {
        let matches = property_type
            .generic_arguments()
            .get_at(0)
            .and_then(|argument| argument.type_argument().ok())
            .and_then(|argument| argument.find_concrete_type())
            .zip(getter.output_element_type_id())
            .is_some_and(|(element, output)| element.type_id() == output);
        return matches;
    }
    let Some(output_type) = context.find_type_by_id(output_type_id).ok().flatten() else {
        return false;
    };
    if matches!(
        (property_type.kind(), output_type.kind()),
        (TypeKind::Text(_), TypeKind::Text(_))
    ) {
        return true;
    }
    let property_element = match property_type.kind() {
        TypeKind::Sequence(_) | TypeKind::Array(_) => property_type.generic_arguments().get_at(0),
        _ => None,
    }
    .and_then(|argument| argument.type_argument().ok())
    .and_then(|argument| argument.find_concrete_type());
    let output_element = match output_type.kind() {
        TypeKind::Slice(_) => output_type.generic_arguments().get_at(0),
        _ => None,
    }
    .and_then(|argument| argument.type_argument().ok())
    .and_then(|argument| argument.find_concrete_type());
    property_element
        .zip(output_element)
        .is_some_and(|(a, b)| a.type_id() == b.type_id())
}

/// Checks whether an accessor type is a text-compatible view of a property.
fn text_accessor_type_compatible(
    property_type: TypeDescriptor<'_>,
    accessor_type_id: TypeId,
    context: &ReflectionContext,
) -> bool {
    if !matches!(property_type.kind(), TypeKind::Text(_)) {
        return false;
    }
    if accessor_type_id == property_type.type_id() {
        return true;
    }
    if property_type.type_id() == TypeId::of::<String>() && accessor_type_id == TypeId::of::<str>() {
        return true;
    }
    context
        .find_type_by_id(accessor_type_id)
        .ok()
        .flatten()
        .is_some_and(|accessor_type| matches!(accessor_type.kind(), TypeKind::Text(_)))
}

/// Checks whether two accessor identities agree for one property type.
fn accessor_types_compatible(
    property_type: Option<TypeDescriptor<'_>>,
    left_type_id: TypeId,
    right_type_id: TypeId,
    context: &ReflectionContext,
) -> bool {
    left_type_id == right_type_id
        || property_type.is_some_and(|property_type| {
            text_accessor_type_compatible(property_type, left_type_id, context)
                && text_accessor_type_compatible(property_type, right_type_id, context)
        })
}

/// Verifies that generated property overlays agree with fields and reflection.
fn validate_properties(
    properties: &[PropertyMetadata],
    fields: &[FieldMetadata],
    descriptor: TypeDescriptor<'_>,
) -> Result<(), PropertyBuildErrors> {
    let mut names = HashSet::with_capacity(properties.len());
    let mut errors = Vec::new();
    for property in properties {
        if property.name().is_empty() || !names.insert(property.name()) {
            errors.push(PropertyBuildError::new(
                PropertyBuildErrorKind::InvalidName,
                property.name(),
            ));
        }
        if property.field().is_none() && property.getter().is_none() && property.setter().is_none() {
            errors.push(PropertyBuildError::new(
                PropertyBuildErrorKind::MissingSource,
                property.name(),
            ));
        }
        if let Some(field) = property.field() {
            let reflected = field.reflect_in(descriptor.context());
            let is_owned_field = contains_field(fields, field);
            if !is_owned_field
                || (!field.is_opaque()
                    && reflected
                        .as_ref()
                        .is_some_and(|reflected| reflected.declaring_type().type_id() != descriptor.type_id()))
            {
                errors.push(PropertyBuildError::new(
                    PropertyBuildErrorKind::ForeignField,
                    property.name(),
                ));
            }
            if is_owned_field && !field.is_opaque() && property.getter().is_none() && property.setter().is_none() {
                let expression_mismatch = property
                    .declared_type_expression_in(descriptor.context())
                    .zip(field.type_expression_data().map(|expression| {
                        qubit_reflect::__private::codegen_v4::expression::type_expression_in(
                            expression,
                            descriptor.context(),
                        )
                    }))
                    .is_some_and(|(declared, field)| declared.display().to_string() != field.display().to_string());
                if expression_mismatch {
                    errors.push(PropertyBuildError::new(
                        PropertyBuildErrorKind::FieldTypeMismatch,
                        property.name(),
                    ));
                }
            }
        }
        if let Some(getter) = property.getter()
            && (getter.target_type_id() != descriptor.type_id()
                || (property.field().is_none()
                    && !property_accessor_expression_matches(
                        property,
                        getter.output_type_in(descriptor.context()),
                        descriptor.context(),
                    ))
                || !getter_type_compatible(
                    property_type_descriptor(property, fields, descriptor.context()),
                    getter,
                    descriptor.context(),
                ))
        {
            errors.push(PropertyBuildError::new(
                PropertyBuildErrorKind::GetterTypeMismatch,
                property.name(),
            ));
        }
        if let Some(setter) = property.setter()
            && (setter.target_type_id() != descriptor.type_id()
                || (property.field().is_none()
                    && property.getter().is_none()
                    && !property_accessor_expression_matches(
                        property,
                        setter.input_type_in(descriptor.context()),
                        descriptor.context(),
                    ))
                || property_type_descriptor(property, fields, descriptor.context()).is_some_and(|value| {
                    !accessor_types_compatible(
                        Some(value),
                        value.type_id(),
                        setter.input_type_id(),
                        descriptor.context(),
                    )
                })
                || property.getter().is_some_and(|getter| {
                    !accessor_types_compatible(
                        property_type_descriptor(property, fields, descriptor.context()),
                        getter.output_type_id(),
                        setter.input_type_id(),
                        descriptor.context(),
                    )
                }))
        {
            errors.push(PropertyBuildError::new(
                PropertyBuildErrorKind::SetterTypeMismatch,
                property.name(),
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(PropertyBuildErrors::new(errors))
    }
}

/// Verifies that role-specific metadata agrees with the reflected type.
fn validate_role(metadata: &TypeMetadata, descriptor: TypeDescriptor<'_>) -> Result<(), AbiViolation> {
    match metadata.role_metadata() {
        RoleMetadata::Entity(role) => validate_identifier(metadata.fields, role.identifier())?,
        RoleMetadata::Projection(role) => validate_identifier(metadata.fields, role.identifier())?,
        RoleMetadata::Model(_) => {}
        RoleMetadata::Value(role) => {
            if let Some(field) = role.transparent_field()
                && (metadata.fields.len() != 1 || !contains_field(metadata.fields, field))
            {
                return Err(abi_violation(
                    "QMM-ABI-011",
                    "a transparent value must reference its only field",
                ));
            }
            if let Some(codec) = role.canonical_codec()
                && codec.source() != crate::metadata::CodecSource::CanonicalValue
            {
                return Err(abi_violation(
                    "QMM-ABI-025",
                    "codec source differs from its metadata position",
                ));
            }
        }
        RoleMetadata::Enum(role) => {
            let reflected = descriptor.variants();
            if role.variants().len() != reflected.len() {
                return Err(abi_violation(
                    "QMM-ABI-012",
                    "enum variant metadata count differs from reflection metadata",
                ));
            }
            let mut defaults = 0;
            let mut canonical_names = HashSet::with_capacity(role.variants().len());
            let mut serialized_names = HashSet::with_capacity(role.variants().len());
            let mut deserialized_names = HashSet::with_capacity(role.variants().len());
            for (index, (variant, reflect)) in role.variants().iter().zip(reflected.iter()).enumerate() {
                if variant.index() != index
                    || reflect.index() != index
                    || reflect.declaring_type().type_id() != descriptor.type_id()
                {
                    return Err(abi_violation(
                        "QMM-ABI-012",
                        "enum variants are not in reflection source order",
                    ));
                }
                if variant.canonical_name().is_empty()
                    || !canonical_names.insert(variant.canonical_name())
                    || !serialized_names.insert(variant.serialized_name())
                    || !deserialized_names.insert(variant.deserialized_name())
                {
                    return Err(abi_violation(
                        "QMM-ABI-024",
                        "enum variant names must be non-empty and unique per namespace",
                    ));
                }
                validate_fields(variant.fields(), reflect.fields(), descriptor, "QMM-ABI-013")?;
                defaults += usize::from(variant.is_default());
            }
            if defaults > 1 {
                return Err(abi_violation(
                    "QMM-ABI-014",
                    "an enum cannot declare more than one default variant",
                ));
            }
        }
    }
    Ok(())
}

/// Verifies that an identifier belongs to the declaring field collection.
fn validate_identifier(fields: &[FieldMetadata], identifier: &FieldMetadata) -> Result<(), AbiViolation> {
    if !contains_field(fields, identifier)
        || !identifier.is_identifier()
        || fields.iter().filter(|field| field.is_identifier()).count() != 1
    {
        return Err(abi_violation(
            "QMM-ABI-010",
            "the role identifier must be an identifier field on the metadata root",
        ));
    }
    Ok(())
}

/// Returns whether `candidate` is one of the declared field overlays.
fn contains_field(fields: &[FieldMetadata], candidate: &FieldMetadata) -> bool {
    fields.iter().any(|field| core::ptr::eq(field, candidate))
}

/// Creates a stable ABI diagnostic for invalid generated metadata.
const fn abi_violation(code: &'static str, message: &'static str) -> AbiViolation {
    AbiViolation::new(code, message)
}
