// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Hidden, versioned ABI consumed by generated model code.

pub use qubit_id;
pub use qubit_redact as redact;
pub use qubit_reflect::__private::__inventory;
pub use qubit_reflect::__private::codegen_v4;
pub use qubit_reflect::Reflect;
pub use qubit_reflect::ReflectImpl;
pub use qubit_reflect::TypeDescriptor;
pub use qubit_reflect::capability::CapabilityDescriptor;
pub use qubit_reflect::descriptor::Reflect;
pub use qubit_reflect::error::TypeMismatch;
pub use qubit_reflect::value::DynamicMut;
pub use qubit_reflect::value::DynamicOwned;
pub use qubit_reflect::value::DynamicRef;
pub use qubit_reflect::value::Local;
pub use serde;

#[path = "private/identifier_accessor.rs"]
mod identifier_accessor;
pub use identifier_accessor::IdentifierAccessor;

pub use crate::metadata::NamedValidationArgument;
pub use crate::metadata::PropertyBorrowed;
pub use crate::metadata::PropertyMutable;
pub use crate::metadata::PropertyOwned;
pub use crate::metadata::Sensitivity;
pub use crate::metadata::ValidationArgument;
pub use crate::reflect_facade::ModelImplFragment;
pub use crate::reflect_facade::ModelImplProvider;
pub use crate::reflect_facade::ModelImplSource;
pub use crate::reflect_facade::ModelMetadataCapability;
pub use crate::reflect_facade::ModelMetadataProvider;

#[path = "private/reflect_codegen.rs"]
mod reflect_codegen;

#[path = "private/model_type_seal.rs"]
mod model_type_seal;
pub use model_type_seal::ModelTypeSeal;

#[path = "private/type_metadata_provider.rs"]
mod type_metadata_provider;
pub use type_metadata_provider::TypeMetadataProvider;

#[path = "private/model_impl_seal.rs"]
mod model_impl_seal;
pub use model_impl_seal::ModelImplSeal;

/// Compile-time assertion helpers shared by the generated-code ABI.
#[doc(hidden)]
#[path = "private/compile_assertions/mod.rs"]
mod compile_assertions;

/// Current generated-code ABI namespace. Role constructors now take an
/// identifier accessor; generated code uses [`v8`] to make that signature
/// change explicit at the runtime/derive boundary.
///
/// All intentionally permanent allocations used by generic metadata are
/// centralized here. Generated code must finish each aggregate through
/// [`v8::GeneratedTypeMetadataBuilder::finish`] so malformed metadata fails at
/// its construction boundary.
#[doc(hidden)]
pub mod v8 {
    use std::any::TypeId;

    use qubit_reflect::__private::codegen_v4::expression::TypeExpressionData;
    #[cfg(feature = "generic")]
    pub use qubit_reflect::capability::CapabilityRegistration;
    pub use qubit_reflect::descriptor::Visibility;

    pub use super::compile_assertions::BorrowedPropertyOutput;
    pub use super::compile_assertions::DecimalConstraintTarget;
    pub use super::compile_assertions::GeneratedTypeMetadataBuilder;
    pub use super::compile_assertions::IdentifierType;
    pub use super::compile_assertions::MapConstraintTarget;
    pub use super::compile_assertions::NestedTypeMetadataProbe;
    pub use super::compile_assertions::OptionalBorrowedPropertyOutput;
    pub use super::compile_assertions::PropertyOutputCompatible;
    pub use super::compile_assertions::SequenceConstraintTarget;
    pub use super::compile_assertions::TemporalConstraintTarget;
    pub use super::compile_assertions::TextConstraintTarget;
    pub use super::compile_assertions::UniqueItemsConstraintTarget;
    pub use super::compile_assertions::VariableLengthSequenceTarget;
    pub use super::compile_assertions::local_property_set;
    pub use super::compile_assertions::model_impl_metadata;
    pub use super::compile_assertions::property_fragment;
    #[cfg(feature = "generic")]
    pub use crate::__qubit_model_register_generic_model_capability as register_generic_model_capability;
    #[cfg(feature = "generic")]
    pub use crate::__qubit_model_register_generic_model_impl_capability as register_generic_model_impl_capability;
    pub use crate::__qubit_model_submit_metadata_capability as submit_model_metadata_capability;
    pub use crate::__qubit_model_submit_model_impl_capability as submit_model_impl_capability;
    pub use crate::metadata::RustTypeReference;
    #[cfg(feature = "generic")]
    pub use crate::reflect_facade::GenericModelMetadataCapability;
    #[cfg(feature = "generic")]
    pub use crate::reflect_facade::GenericModelMetadataProvider;
    pub use crate::reflect_facade::ModelImplCapability;
    pub use crate::reflect_facade::all_model_impl_fragments;
    #[cfg(feature = "generic")]
    pub use crate::reflect_facade::generic_model_capability;
    pub use crate::reflect_facade::model_capability;
    pub use crate::reflect_facade::model_impl_capability_registration;
    pub use crate::reflect_facade::model_impl_registration;
    pub use crate::reflect_facade::model_metadata_registration;

    /// Builds one field metadata overlay from generated declarations.
    ///
    /// # Parameters
    ///
    /// - `owner`: Runtime identity of the type that owns the field.
    /// - `variant`: Optional enum variant index containing the field.
    /// - `index`: Position of the field in its declaration.
    /// - `name`: Optional source field name.
    /// - `visibility`: Visibility recorded by reflection.
    /// - `expression`: Type expression used to resolve the field type.
    /// - `attributes`: Field attributes supplied by the derive.
    /// - `constraints`: Constraints declared for the field.
    /// - `validators`: Validators declared for the field.
    /// - `serde`: Serialization behavior recorded for the field.
    ///
    /// # Returns
    ///
    /// Field metadata with the supplied declaration semantics and no opaque
    /// type override.
    #[doc(hidden)]
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn field_metadata(
        owner: TypeId,
        variant: Option<usize>,
        index: usize,
        name: Option<&'static str>,
        visibility: Visibility,
        expression: &'static TypeExpressionData,
        attributes: &'static [crate::metadata::FieldAttributeMetadata],
        constraints: &'static [crate::metadata::ConstraintMetadata],
        validators: &'static [crate::metadata::ValidatorMetadata],
        serde: &'static crate::metadata::SerdeFieldMetadata,
    ) -> crate::metadata::FieldMetadata {
        field_metadata_with_opaque_type_id(
            owner,
            variant,
            index,
            name,
            visibility,
            expression,
            None,
            attributes,
            constraints,
            validators,
            serde,
        )
    }

    /// Builds one field overlay while retaining an optional exact opaque type.
    ///
    /// # Parameters
    ///
    /// - `owner`: Runtime identity of the type that owns the field.
    /// - `variant`: Optional enum variant index containing the field.
    /// - `index`: Position of the field in its declaration.
    /// - `name`: Optional source field name.
    /// - `visibility`: Visibility recorded by reflection.
    /// - `expression`: Type expression used to resolve the field type.
    /// - `opaque_type_id`: Exact runtime type identity when type syntax is
    ///   opaque; `None` leaves resolution to the expression.
    /// - `attributes`: Field attributes supplied by the derive.
    /// - `constraints`: Constraints declared for the field.
    /// - `validators`: Validators declared for the field.
    /// - `serde`: Serialization behavior recorded for the field.
    ///
    /// # Returns
    ///
    /// Field metadata retaining the optional opaque type identity.
    #[doc(hidden)]
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn field_metadata_with_opaque_type_id(
        owner: TypeId,
        variant: Option<usize>,
        index: usize,
        name: Option<&'static str>,
        visibility: Visibility,
        expression: &'static TypeExpressionData,
        opaque_type_id: Option<TypeId>,
        attributes: &'static [crate::metadata::FieldAttributeMetadata],
        constraints: &'static [crate::metadata::ConstraintMetadata],
        validators: &'static [crate::metadata::ValidatorMetadata],
        serde: &'static crate::metadata::SerdeFieldMetadata,
    ) -> crate::metadata::FieldMetadata {
        crate::metadata::FieldMetadata::with_semantics(
            owner,
            variant,
            index,
            name,
            visibility,
            expression,
            opaque_type_id,
            attributes,
            constraints,
            validators,
            serde,
        )
    }

    /// Builds a semantic overlay for one generic declaration field.
    ///
    /// # Parameters
    ///
    /// - `definition_path`: Stable path identifying the generic definition.
    /// - `index`: Position of the field in its declaration.
    /// - `variant`: Optional enum variant index containing the field.
    /// - `name`: Optional source field name.
    /// - `visibility`: Visibility recorded by reflection.
    /// - `expression`: Type expression used to resolve the field type.
    /// - `variant_inherited`: Whether the field was inherited from a variant.
    /// - `attributes`: Field attributes supplied by the derive.
    /// - `constraints`: Constraints declared for the field.
    /// - `validators`: Validators declared for the field.
    /// - `serde`: Serialization behavior recorded for the field.
    ///
    /// # Returns
    ///
    /// Field metadata associated with the generic definition.
    #[doc(hidden)]
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn generic_field_metadata(
        definition_path: &'static str,
        index: usize,
        variant: Option<usize>,
        name: Option<&'static str>,
        visibility: Visibility,
        expression: &'static TypeExpressionData,
        variant_inherited: bool,
        attributes: &'static [crate::metadata::FieldAttributeMetadata],
        constraints: &'static [crate::metadata::ConstraintMetadata],
        validators: &'static [crate::metadata::ValidatorMetadata],
        serde: &'static crate::metadata::SerdeFieldMetadata,
    ) -> crate::metadata::FieldMetadata {
        crate::metadata::FieldMetadata::with_definition_semantics(
            definition_path,
            index,
            variant,
            name,
            visibility,
            expression,
            variant_inherited,
            attributes,
            constraints,
            validators,
            serde,
        )
    }

    /// Builds one merged property metadata value.
    ///
    /// # Parameters
    ///
    /// - `name`: Logical property name.
    /// - `type_expression`: `Some` supplies a declared type expression; `None`
    ///   leaves the property type to its field or accessor metadata.
    /// - `field`: `Some` associates a backing field; `None` represents a
    ///   property without a field declaration.
    /// - `getter`: `Some` associates a getter; `None` means no getter exists.
    /// - `setter`: `Some` associates a setter; `None` means no setter exists.
    ///
    /// # Returns
    ///
    /// The merged property metadata.
    #[doc(hidden)]
    #[must_use]
    pub const fn property_metadata(
        name: &'static str,
        type_expression: Option<&'static TypeExpressionData>,
        field: Option<&'static crate::metadata::FieldMetadata>,
        getter: Option<&'static crate::metadata::GetterMetadata>,
        setter: Option<&'static crate::metadata::SetterMetadata>,
    ) -> crate::metadata::PropertyMetadata {
        crate::metadata::PropertyMetadata::new(name, type_expression, field, getter, setter)
    }

    /// Builds entity-role metadata for an identifier field.
    ///
    /// # Parameters
    ///
    /// - `identifier`: Field metadata for the identifier field.
    /// - `identifier_accessor`: Generated callback that reads the identifier.
    ///
    /// # Returns
    ///
    /// Entity role metadata using the supplied field and accessor.
    #[doc(hidden)]
    #[must_use]
    pub const fn entity_role(
        identifier: &'static crate::metadata::FieldMetadata,
        identifier_accessor: crate::__private::IdentifierAccessor,
    ) -> crate::metadata::RoleMetadata {
        crate::metadata::RoleMetadata::Entity(crate::metadata::EntityMetadata::new(identifier, identifier_accessor))
    }

    /// Builds projection-role metadata and its optional source target.
    ///
    /// # Parameters
    ///
    /// - `identifier`: Field metadata for the projection identifier.
    /// - `source`: `Some` supplies the declared entity target; `None` records a
    ///   projection without a declared source target.
    /// - `identifier_accessor`: Generated callback that reads the identifier.
    ///
    /// # Returns
    ///
    /// Projection role metadata with the optional source target.
    #[doc(hidden)]
    #[must_use]
    pub const fn projection_role(
        identifier: &'static crate::metadata::FieldMetadata,
        source: Option<&'static crate::metadata::DeclaredEntityTarget>,
        identifier_accessor: crate::__private::IdentifierAccessor,
    ) -> crate::metadata::RoleMetadata {
        crate::metadata::RoleMetadata::Projection(crate::metadata::ProjectionMetadata::new(
            identifier,
            source,
            identifier_accessor,
        ))
    }

    /// Builds metadata for a general model role.
    ///
    /// # Returns
    ///
    /// Metadata identifying the type as a general model.
    #[doc(hidden)]
    #[must_use]
    pub const fn model_role() -> crate::metadata::RoleMetadata {
        crate::metadata::RoleMetadata::Model(crate::metadata::ModelMetadata)
    }

    /// Builds value-role metadata.
    ///
    /// # Parameters
    ///
    /// - `transparent_field`: `Some` identifies the transparent field; `None`
    ///   means the value has no transparent field.
    /// - `canonical_codec`: `Some` supplies the canonical codec; `None` means
    ///   no canonical codec was declared.
    ///
    /// # Returns
    ///
    /// Value role metadata with the supplied optional field and codec.
    #[doc(hidden)]
    #[must_use]
    pub const fn value_role(
        transparent_field: Option<&'static crate::metadata::FieldMetadata>,
        canonical_codec: Option<&'static crate::metadata::CodecMetadata>,
    ) -> crate::metadata::RoleMetadata {
        crate::metadata::RoleMetadata::Value(crate::metadata::ValueMetadata::new(transparent_field, canonical_codec))
    }

    /// Builds metadata for one generated enum variant.
    ///
    /// # Parameters
    ///
    /// - `owner`: Runtime identity of the enum type.
    /// - `index`: Variant position in the declaration.
    /// - `rust_name`: Source-language variant name.
    /// - `canonical_name`: Canonical model name for the variant.
    /// - `serialized_name`: Name used during serialization.
    /// - `deserialized_name`: Name accepted during deserialization.
    /// - `fields`: Field metadata belonging to the variant.
    /// - `default`: Whether this variant is the declared default.
    ///
    /// # Returns
    ///
    /// Metadata describing the generated enum variant.
    #[doc(hidden)]
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn enum_variant_metadata(
        owner: TypeId,
        index: usize,
        rust_name: &'static str,
        canonical_name: &'static str,
        serialized_name: &'static str,
        deserialized_name: &'static str,
        fields: &'static [crate::metadata::FieldMetadata],
        default: bool,
    ) -> crate::metadata::EnumVariantMetadata {
        crate::metadata::EnumVariantMetadata::new(
            owner,
            index,
            rust_name,
            canonical_name,
            serialized_name,
            deserialized_name,
            fields,
            default,
        )
    }

    /// Builds metadata for one generic enum declaration variant.
    ///
    /// # Parameters
    ///
    /// - `definition_path`: Stable path identifying the generic definition.
    /// - `index`: Variant position in the declaration.
    /// - `rust_name`: Source-language variant name.
    /// - `canonical_name`: Canonical model name for the variant.
    /// - `serialized_name`: Name used during serialization.
    /// - `deserialized_name`: Name accepted during deserialization.
    /// - `fields`: Field metadata belonging to the variant.
    /// - `default`: Whether this variant is the declared default.
    ///
    /// # Returns
    ///
    /// Variant metadata associated with the generic definition.
    #[doc(hidden)]
    #[must_use]
    #[cfg(feature = "generic")]
    #[allow(clippy::too_many_arguments)]
    pub const fn generic_enum_variant_metadata(
        definition_path: &'static str,
        index: usize,
        rust_name: &'static str,
        canonical_name: &'static str,
        serialized_name: &'static str,
        deserialized_name: &'static str,
        fields: &'static [crate::metadata::FieldMetadata],
        default: bool,
    ) -> crate::metadata::EnumVariantMetadata {
        crate::metadata::EnumVariantMetadata::from_definition(
            definition_path,
            index,
            rust_name,
            canonical_name,
            serialized_name,
            deserialized_name,
            fields,
            default,
        )
    }

    /// Builds enum-role metadata from generated variants.
    ///
    /// # Parameters
    ///
    /// - `variants`: Static metadata for the enum's declared variants.
    ///
    /// # Returns
    ///
    /// Role metadata containing the supplied enum variants.
    #[doc(hidden)]
    #[must_use]
    pub const fn enum_role(variants: &'static [crate::metadata::EnumVariantMetadata]) -> crate::metadata::RoleMetadata {
        crate::metadata::RoleMetadata::Enum(crate::metadata::EnumMetadata::new(variants))
    }

    /// Builds metadata for one generic model definition.
    ///
    /// # Parameters
    ///
    /// - `model_id`: Optional model identifier convertible to its stored form.
    /// - `role`: Role represented by the generic definition.
    /// - `definition_path`: Stable source path for the definition.
    /// - `fields`: Static field metadata for the definition.
    /// - `variants`: Static enum variant metadata for the definition.
    ///
    /// # Returns
    ///
    /// Generic model metadata containing the declaration data.
    #[doc(hidden)]
    #[must_use]
    #[cfg(feature = "generic")]
    pub fn generic_model_metadata(
        model_id: impl Into<Option<crate::metadata::ModelId>>,
        role: crate::metadata::ModelRole,
        definition_path: &'static str,
        fields: &'static [crate::metadata::FieldMetadata],
        variants: &'static [crate::metadata::EnumVariantMetadata],
    ) -> crate::generic::GenericModelMetadata {
        crate::generic::GenericModelMetadata::new(model_id.into(), role, definition_path, fields, variants)
    }

    /// Leaks a generated value for static metadata storage.
    ///
    /// The value is intentionally kept for the lifetime of the process so
    /// generated metadata can store a stable reference.
    ///
    /// # Type Parameters
    ///
    /// - `T`: Value type retained in static metadata.
    ///
    /// # Parameters
    ///
    /// - `value`: Value to retain permanently.
    ///
    /// # Returns
    ///
    /// A process-lifetime shared reference to the value.
    #[doc(hidden)]
    #[must_use]
    pub fn leak<T: 'static>(value: T) -> &'static T {
        Box::leak(Box::new(value))
    }

    /// Leaks generated values as a static slice for metadata storage.
    ///
    /// The allocation is intentionally kept for the lifetime of the process
    /// so generated metadata can store a stable slice.
    ///
    /// # Type Parameters
    ///
    /// - `T`: Element type retained in static metadata.
    ///
    /// # Parameters
    ///
    /// - `values`: Elements to retain permanently.
    ///
    /// # Returns
    ///
    /// A process-lifetime shared slice containing the supplied elements.
    #[doc(hidden)]
    #[must_use]
    pub fn leak_slice<T: 'static>(values: Vec<T>) -> &'static [T] {
        Box::leak(values.into_boxed_slice())
    }

    #[doc(hidden)]
    pub use crate::__qubit_model_with_generic_feature as with_generic_feature;
}
