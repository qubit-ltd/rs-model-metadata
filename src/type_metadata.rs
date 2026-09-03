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

use qubit_reflect::ConcreteGenericDescriptor;
use qubit_reflect::FieldDescriptor;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::OpaqueTypeDescriptor;
use qubit_reflect::descriptor::TextKind;
use qubit_reflect::descriptor::TypeRef;

pub use self::enum_metadata::EnumMetadata;
pub use self::enum_variant_metadata::EnumVariantMetadata;
pub use self::has_type_metadata::HasTypeMetadata;
use crate::AbiViolation;
use crate::ConstraintMetadata;
use crate::EntityMetadata;
use crate::FieldAttributeMetadata;
use crate::FieldMetadata;
use crate::GenericModelMetadata;
use crate::LocalPropertySet;
use crate::ModelId;
use crate::ModelMetadata;
use crate::ModelRole;
use crate::ProjectionMetadata;
use crate::PropertyBuildError;
use crate::PropertyBuildErrorKind;
use crate::PropertyBuildErrors;
use crate::PropertyFragment;
use crate::PropertyMetadata;
use crate::RoleMetadata;
use crate::SelectorPosition;
use crate::ValueMetadata;

/// Domain semantics for one concrete reflected Rust type.
#[derive(Clone, Copy, Debug)]
pub struct TypeMetadata {
    /// The reflection descriptor that owns this metadata overlay.
    descriptor: &'static TypeDescriptor,
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
    generic_definition: Option<&'static GenericModelMetadata>,
}

impl TypeMetadata {
    /// Creates generated role-aware metadata over an existing descriptor root.
    #[doc(hidden)]
    #[must_use]
    pub(crate) const fn new(
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
            properties: LocalPropertySet::new(&[]),
            property_fragments: &[],
            generic_definition: None,
        }
    }

    /// Adds generated property metadata to this immutable overlay.
    #[doc(hidden)]
    #[must_use]
    pub(crate) const fn with_properties(mut self, properties: &'static [PropertyMetadata]) -> Self {
        self.properties = LocalPropertySet::new(properties);
        self
    }

    /// Adds generated field property fragments to this immutable overlay.
    #[doc(hidden)]
    #[must_use]
    pub(crate) const fn with_property_fragments(mut self, fragments: &'static [PropertyFragment]) -> Self {
        self.property_fragments = fragments;
        self
    }

    /// Records the generic definition that produced this concrete instance.
    #[doc(hidden)]
    #[must_use]
    pub(crate) const fn with_generic_definition(mut self, definition: &'static GenericModelMetadata) -> Self {
        self.generic_definition = Some(definition);
        self
    }

    /// Returns the static metadata generated for `T`.
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
    /// # Errors
    ///
    /// Returns a structured violation when generated metadata disagrees with
    /// the unique reflection descriptor for `T`.
    pub fn try_of<T: HasTypeMetadata>() -> Result<&'static Self, AbiViolation> {
        let metadata = <T as crate::__private::TypeMetadataProvider>::__type_metadata();
        metadata.validate_for::<T>()?;
        Ok(metadata)
    }

    /// Returns the unique reflection descriptor root.
    #[must_use]
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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

    /// Returns unmerged field/getter/setter declarations in source order.
    #[must_use]
    pub fn property_fragments(&'static self) -> &'static [PropertyFragment] {
        self.descriptor
            .get_capability(crate::reflect_facade::model_impl_key())
            .map_or(self.property_fragments, |provider| provider().fragments())
    }

    /// Returns locally merged field and method properties.
    ///
    /// # Errors
    ///
    /// Returns deterministic property assembly errors when an independent
    /// `ModelImpl` block disagrees with the model's reflected fields.
    pub fn try_properties(&'static self) -> Result<&'static LocalPropertySet, &'static PropertyBuildErrors> {
        self.descriptor
            .get_capability(crate::reflect_facade::model_impl_key())
            .map_or(Ok(&self.properties), |provider| provider().try_properties())
    }

    /// Finds a locally merged property by its public name.
    ///
    /// # Errors
    ///
    /// Returns the same assembly errors as [`Self::try_properties`].
    pub fn try_property(
        &'static self,
        name: &str,
    ) -> Result<Option<&'static PropertyMetadata>, &'static PropertyBuildErrors> {
        self.try_properties().map(|properties| properties.property(name))
    }

    /// Returns the generic model template that produced this metadata.
    #[must_use]
    #[inline(always)]
    pub const fn generic_definition(&self) -> Option<&'static GenericModelMetadata> {
        self.generic_definition
    }

    /// Returns concrete reflection substitutions for a generic instance.
    #[must_use]
    #[inline(always)]
    pub const fn concrete_generic(&self) -> Option<&'static ConcreteGenericDescriptor> {
        self.descriptor.concrete_generic()
    }

    /// Returns the model role tag.
    #[must_use]
    #[inline(always)]
    pub const fn role(&self) -> ModelRole {
        self.role.role()
    }

    /// Returns the role-specific metadata payload.
    #[must_use]
    #[inline(always)]
    pub const fn role_metadata(&self) -> &'static RoleMetadata {
        self.role
    }

    /// Returns Entity metadata when this is an Entity.
    #[must_use]
    #[inline(always)]
    pub const fn as_entity(&self) -> Option<&'static EntityMetadata> {
        match self.role {
            RoleMetadata::Entity(value) => Some(value),
            _ => None,
        }
    }

    /// Returns Projection metadata when this is a Projection.
    #[must_use]
    #[inline(always)]
    pub const fn as_projection(&self) -> Option<&'static ProjectionMetadata> {
        match self.role {
            RoleMetadata::Projection(value) => Some(value),
            _ => None,
        }
    }

    /// Returns Model metadata when this is a Model.
    #[must_use]
    #[inline(always)]
    pub const fn as_model(&self) -> Option<&'static ModelMetadata> {
        match self.role {
            RoleMetadata::Model(value) => Some(value),
            _ => None,
        }
    }

    /// Returns Enum metadata when this is an Enum.
    #[must_use]
    #[inline(always)]
    pub const fn as_enum(&self) -> Option<&'static EnumMetadata> {
        match self.role {
            RoleMetadata::Enum(value) => Some(value),
            _ => None,
        }
    }

    /// Returns Value metadata when this is a Value.
    #[must_use]
    #[inline(always)]
    pub const fn as_value(&self) -> Option<&'static ValueMetadata> {
        match self.role {
            RoleMetadata::Value(value) => Some(value),
            _ => None,
        }
    }

    /// Verifies that generated metadata is anchored to `T` and internally
    /// consistent before it crosses the public ABI boundary.
    #[doc(hidden)]
    pub fn assert_valid_for<T: 'static>(&self) {
        self.validate_for::<T>().unwrap_or_else(|error| panic!("{error}"));
    }

    /// Checks that generated metadata is anchored to `T`.
    #[doc(hidden)]
    pub fn validate_for<T: 'static>(&self) -> Result<(), AbiViolation> {
        if self.descriptor.type_id() != TypeId::of::<T>() {
            return Err(abi_violation(
                "QMM-ABI-001",
                "metadata descriptor does not describe the requested Rust type",
            ));
        }
        self.validate_descriptor(self.descriptor)
    }

    /// Verifies that this metadata remains attached to `descriptor`.
    ///
    /// # Panics
    ///
    /// Panics with a stable ABI diagnostic when metadata and reflection differ.
    pub(crate) fn assert_valid_descriptor(&self, descriptor: &TypeDescriptor) {
        self.validate_descriptor(descriptor)
            .unwrap_or_else(|error| panic!("{error}"));
    }

    /// Checks that this metadata remains attached to `descriptor`.
    pub(crate) fn validate_descriptor(&self, descriptor: &TypeDescriptor) -> Result<(), AbiViolation> {
        if !core::ptr::eq(self.descriptor, descriptor) {
            return Err(abi_violation(
                "QMM-ABI-002",
                "metadata is attached to a different reflection root",
            ));
        }

        if !matches!(self.role, RoleMetadata::Enum(_)) {
            validate_fields(self.fields, descriptor.fields(), descriptor, "QMM-ABI-003")?;
        }
        if self.model_id.is_some() && self.generic_definition.is_some() {
            return Err(abi_violation(
                "QMM-ABI-022",
                "concrete metadata cannot own both a model ID and a generic definition",
            ));
        }
        if let Some(definition) = self.generic_definition
            && (self.descriptor.concrete_generic().is_none() || definition.role() != self.role())
        {
            return Err(abi_violation(
                "QMM-ABI-022",
                "generic metadata does not match its concrete instance",
            ));
        }
        if validate_properties(self.properties.properties(), self.fields, descriptor).is_err() {
            return Err(abi_violation(
                "QMM-ABI-004",
                "generated field properties are internally inconsistent",
            ));
        }
        validate_role(self, descriptor)
    }

    /// Verifies a property overlay supplied by a generated capability.
    #[doc(hidden)]
    pub fn validate_properties(&self, properties: &[PropertyMetadata]) -> Result<(), PropertyBuildErrors> {
        validate_properties(properties, self.fields, self.descriptor)
    }
}

/// Verifies that field overlays mirror the reflection descriptor exactly.
fn validate_fields(
    metadata: &[FieldMetadata],
    reflected: &[FieldDescriptor],
    descriptor: &TypeDescriptor,
    code: &'static str,
) -> Result<(), AbiViolation> {
    if metadata.len() != reflected.len() {
        return Err(abi_violation(
            code,
            "field metadata count differs from reflection metadata",
        ));
    }
    for (index, (field, reflect)) in metadata.iter().zip(reflected).enumerate() {
        if field.index() != index
            || !core::ptr::eq(field.reflect(), reflect)
            || !core::ptr::eq(field.reflect().declaring_type(), descriptor)
        {
            return Err(abi_violation(code, "field metadata is not in reflection source order"));
        }
        validate_field_semantics(field)?;
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
fn validate_field_semantics(field: &FieldMetadata) -> Result<(), AbiViolation> {
    let mut singleton_counts = [0_u8; 9];
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
    validate_constraint_kinds(field.constraints(), field.type_ref(), true)?;
    validate_validators(field.validators())?;
    validate_codec(
        field.codec(),
        field_codec_type_id(field.type_ref()),
        crate::CodecSource::Field,
    )?;
    Ok(())
}

/// Verifies that validator declarations have non-empty, unique dependencies.
fn validate_validators(validators: &[crate::ValidatorMetadata]) -> Result<(), AbiViolation> {
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
    type_ref: &TypeRef,
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
                    validate_selector(selector, SelectorPosition::Element, type_ref)?;
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
                        validate_selector(selector, position, type_ref)?;
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
    selector: &crate::SelectorMetadata,
    position: SelectorPosition,
    type_ref: &TypeRef,
) -> Result<(), AbiViolation> {
    if selector.position() != position {
        return Err(abi_violation(
            "QMM-ABI-020",
            "selector has the wrong structural position",
        ));
    }
    let Some(selected_type) = selector_type_ref(type_ref, position) else {
        return Err(abi_violation(
            "QMM-ABI-020",
            "selector position is incompatible with the field type",
        ));
    };
    validate_constraint_kinds(selector.constraints(), selected_type, false)?;
    validate_validators(selector.validators())?;
    validate_codec(
        selector.codec(),
        type_ref_id(selected_type),
        crate::CodecSource::Selector(position),
    )
}

/// Verifies that a codec declaration is valid for its expected type and source.
fn validate_codec(
    codec: Option<&crate::CodecMetadata>,
    expected_type: Option<TypeId>,
    source: crate::CodecSource,
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
    if let (crate::CodecReference::RustType(descriptor), Some(expected_type)) = (codec.codec(), expected_type)
        && descriptor.value_type_id() != expected_type
    {
        return Err(abi_violation(
            "QMM-ABI-025",
            "codec value type differs from its metadata position",
        ));
    }
    Ok(())
}

/// Returns the codec-compatible type ID for a field type reference.
fn field_codec_type_id(type_ref: &TypeRef) -> Option<TypeId> {
    let Some(descriptor) = type_ref.as_resolved() else {
        return type_ref_id(type_ref);
    };
    descriptor
        .as_optional()
        .and_then(|view| type_ref_id(view.element_type()))
        .or_else(|| Some(descriptor.type_id()))
}

/// Returns the nested type reference selected by a collection position.
fn selector_type_ref(type_ref: &TypeRef, position: SelectorPosition) -> Option<&'static TypeRef> {
    let descriptor = transparent_descriptor(type_ref.as_resolved()?)?;
    match position {
        SelectorPosition::Element => descriptor
            .as_sequence()
            .map(|view| view.element_type())
            .or_else(|| descriptor.as_set().map(|view| view.element_type()))
            .or_else(|| descriptor.as_array().map(|view| view.element_type()))
            .or_else(|| descriptor.as_slice().map(|view| view.element_type())),
        SelectorPosition::MapKey => descriptor.as_map().map(|view| view.key_type()),
        SelectorPosition::MapValue => descriptor.as_map().map(|view| view.value_type()),
    }
}

/// Returns the innermost descriptor through transparent smart-pointer layers.
fn transparent_descriptor(mut descriptor: &'static TypeDescriptor) -> Option<&'static TypeDescriptor> {
    loop {
        let element = descriptor
            .as_optional()
            .map(|view| view.element_type())
            .or_else(|| descriptor.as_smart_pointer().map(|view| view.pointee_type()));
        let Some(element) = element else {
            return Some(descriptor);
        };
        descriptor = element.as_resolved()?;
    }
}

/// Verifies that generated property overlays agree with fields and reflection.
fn validate_properties(
    properties: &[PropertyMetadata],
    fields: &[FieldMetadata],
    descriptor: &TypeDescriptor,
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
            if !contains_field(fields, field) || !core::ptr::eq(field.reflect().declaring_type(), descriptor) {
                errors.push(PropertyBuildError::new(
                    PropertyBuildErrorKind::ForeignField,
                    property.name(),
                ));
            }
            if !type_refs_equal(property.type_ref(), field.type_ref()) {
                errors.push(PropertyBuildError::new(
                    PropertyBuildErrorKind::FieldTypeMismatch,
                    property.name(),
                ));
            }
        }
        if let Some(getter) = property.getter()
            && (getter.target_type_id() != descriptor.type_id()
                || !getter_type_compatible(property.type_ref(), getter.output_type()))
        {
            errors.push(PropertyBuildError::new(
                PropertyBuildErrorKind::GetterTypeMismatch,
                property.name(),
            ));
        }
        if let Some(setter) = property.setter()
            && (setter.target_type_id() != descriptor.type_id()
                || !type_refs_equal(property.type_ref(), setter.input_type())
                || type_ref_id(property.type_ref()).is_some_and(|id| id != setter.input_type_id()))
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
fn validate_role(metadata: &TypeMetadata, descriptor: &TypeDescriptor) -> Result<(), AbiViolation> {
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
            if let Some(codec) = role.canonical_codec() {
                if codec.source() != crate::CodecSource::CanonicalValue {
                    return Err(abi_violation(
                        "QMM-ABI-025",
                        "codec source differs from its metadata position",
                    ));
                }
                if let crate::CodecReference::RustType(codec) = codec.codec()
                    && codec.value_type_id() != metadata.type_id()
                {
                    return Err(abi_violation(
                        "QMM-ABI-023",
                        "canonical codec value type differs from its Value type",
                    ));
                }
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
            for (index, (variant, reflect)) in role.variants().iter().zip(reflected).enumerate() {
                if variant.index() != index
                    || !core::ptr::eq(variant.reflect(), reflect)
                    || !core::ptr::eq(variant.reflect().declaring_type(), descriptor)
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

/// Returns the exact type ID for a resolved type reference.
fn type_ref_id(type_ref: &TypeRef) -> Option<TypeId> {
    type_ref
        .as_resolved()
        .map(TypeDescriptor::type_id)
        .or_else(|| type_ref.as_opaque().map(OpaqueTypeDescriptor::type_id))
}

/// Returns whether two type references resolve to the same exact type.
fn type_refs_equal(left: &TypeRef, right: &TypeRef) -> bool {
    match (type_ref_id(left), type_ref_id(right)) {
        (Some(left), Some(right)) => left == right,
        (None, None) => left.as_symbolic() == right.as_symbolic(),
        _ => false,
    }
}

/// Returns whether a getter output can satisfy its declared property type.
fn getter_type_compatible(property: &TypeRef, output: &TypeRef) -> bool {
    if type_refs_equal(property, output) {
        return true;
    }
    let (Some(property), Some(output)) = (property.as_resolved(), output.as_resolved()) else {
        return false;
    };
    if matches!(
        (
            property.as_text().map(|view| view.kind()),
            output.as_text().map(|view| view.kind())
        ),
        (Some(TextKind::String), Some(TextKind::Str))
    ) {
        return true;
    }
    let property_element = property
        .as_sequence()
        .map(|view| view.element_type())
        .or_else(|| property.as_array().map(|view| view.element_type()));
    let output_element = output.as_slice().map(|view| view.element_type());
    property_element
        .zip(output_element)
        .is_some_and(|(property, output)| type_refs_equal(property, output))
}

/// Creates a stable ABI diagnostic for invalid generated metadata.
const fn abi_violation(code: &'static str, message: &'static str) -> AbiViolation {
    AbiViolation::new(code, message)
}
