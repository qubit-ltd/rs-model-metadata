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

/// Typed accessor used only for the identifier field of Entity and Projection
/// metadata.
#[doc(hidden)]
pub type IdentifierAccessor =
    for<'a> fn(DynamicRef<'a, Local>) -> Result<&'a qubit_id::Id, qubit_reflect::error::TypeMismatch>;

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

/// Marker implemented only by code generated from a model-role macro.
#[doc(hidden)]
pub trait ModelTypeSeal {}

/// Generated provider kept out of the public model bound.
#[doc(hidden)]
pub trait TypeMetadataProvider {
    /// Returns the generated metadata for the implementing model type.
    fn __type_metadata() -> &'static crate::metadata::TypeMetadata;
}

/// Marker implemented once by each generated `ModelImpl` block.
#[doc(hidden)]
pub trait ModelImplSeal {}

/// Compile-time assertion helpers shared by the generated-code ABI.
#[doc(hidden)]
mod compile_assertions {
    use core::marker::PhantomData;

    use qubit_id::Id;

    /// Private sealing boundary for generated compile-time assertions.
    mod sealed {
        use super::Id;

        /// Prevents downstream crates from extending exact model type roles.
        pub trait IdentifierType {}

        impl IdentifierType for Id {}
    }

    /// Marks the exact identifier type accepted by Entity and Projection.
    ///
    /// Type aliases of [`qubit_id::Id`] satisfy this bound because aliases do
    /// not create a distinct Rust type. Wrappers and containers do not.
    #[doc(hidden)]
    pub trait IdentifierType: sealed::IdentifierType {}

    impl IdentifierType for Id {}

    /// Exposes the element selected by sequence constraints without relying
    /// on a Rust type's source spelling.
    #[doc(hidden)]
    pub trait SequenceConstraintTarget {
        type Element: 'static;
    }

    /// Marks sequence shapes whose length may be constrained by a range.
    #[doc(hidden)]
    pub trait VariableLengthSequenceTarget: SequenceConstraintTarget {}

    /// Marks sequence shapes for which `unique_items` is not redundant.
    #[doc(hidden)]
    pub trait UniqueItemsConstraintTarget: SequenceConstraintTarget {}

    /// Exposes the key and value selected by map constraints.
    #[doc(hidden)]
    pub trait MapConstraintTarget {
        type Key: 'static;
        type Value: 'static;
    }

    /// Marks values accepted by text constraints.
    #[doc(hidden)]
    pub trait TextConstraintTarget {}

    /// Marks exact decimal values accepted by decimal and money constraints.
    #[doc(hidden)]
    pub trait DecimalConstraintTarget {}

    /// Marks temporal values accepted by time constraints.
    #[doc(hidden)]
    pub trait TemporalConstraintTarget {}

    impl TextConstraintTarget for str {}
    impl TextConstraintTarget for String {}

    macro_rules! decimal_target {
        ($($type:ty),+ $(,)?) => {
            $(impl DecimalConstraintTarget for $type {})+
        };
    }

    decimal_target!(
        i8,
        i16,
        i32,
        i64,
        i128,
        isize,
        u8,
        u16,
        u32,
        u64,
        u128,
        usize,
        bigdecimal::BigDecimal
    );

    impl<Tz: chrono::TimeZone + 'static> TemporalConstraintTarget for chrono::DateTime<Tz> {}
    impl TemporalConstraintTarget for chrono::NaiveDate {}
    impl TemporalConstraintTarget for chrono::NaiveDateTime {}
    impl TemporalConstraintTarget for chrono::NaiveTime {}

    macro_rules! sequence_target {
        ($($container:ty),+ $(,)?) => {
            $(impl<T: 'static> SequenceConstraintTarget for $container {
                type Element = T;
            })+
        };
    }

    sequence_target!(
        [T],
        Vec<T>,
        std::collections::VecDeque<T>,
        std::collections::LinkedList<T>,
        std::collections::BinaryHeap<T>,
        std::collections::HashSet<T>,
        std::collections::BTreeSet<T>,
    );

    macro_rules! variable_sequence_target {
        ($($container:ty),+ $(,)?) => {
            $(impl<T: 'static> VariableLengthSequenceTarget for $container {})+
        };
    }

    variable_sequence_target!(
        [T],
        Vec<T>,
        std::collections::VecDeque<T>,
        std::collections::LinkedList<T>,
        std::collections::BinaryHeap<T>,
        std::collections::HashSet<T>,
        std::collections::BTreeSet<T>,
    );

    macro_rules! unique_items_target {
        ($($container:ty),+ $(,)?) => {
            $(impl<T: 'static> UniqueItemsConstraintTarget for $container {})+
        };
    }

    unique_items_target!(
        [T],
        Vec<T>,
        std::collections::VecDeque<T>,
        std::collections::LinkedList<T>,
        std::collections::BinaryHeap<T>,
    );

    impl<T: 'static, const N: usize> UniqueItemsConstraintTarget for [T; N] {}

    impl<T: 'static, const N: usize> SequenceConstraintTarget for [T; N] {
        type Element = T;
    }

    impl<K: 'static, V: 'static, S> MapConstraintTarget for std::collections::HashMap<K, V, S> {
        type Key = K;
        type Value = V;
    }

    impl<K: 'static, V: 'static> MapConstraintTarget for std::collections::BTreeMap<K, V> {
        type Key = K;
        type Value = V;
    }

    impl<T: SequenceConstraintTarget> SequenceConstraintTarget for Option<T> {
        type Element = T::Element;
    }
    impl<T: VariableLengthSequenceTarget> VariableLengthSequenceTarget for Option<T> {}
    impl<T: UniqueItemsConstraintTarget> UniqueItemsConstraintTarget for Option<T> {}

    impl<T: MapConstraintTarget> MapConstraintTarget for Option<T> {
        type Key = T::Key;
        type Value = T::Value;
    }

    impl<T: TextConstraintTarget> TextConstraintTarget for Option<T> {}
    impl<T: DecimalConstraintTarget> DecimalConstraintTarget for Option<T> {}
    impl<T: TemporalConstraintTarget> TemporalConstraintTarget for Option<T> {}

    macro_rules! transparent_target {
        ($($wrapper:ty),+ $(,)?) => {
            $(
                impl<T: SequenceConstraintTarget + ?Sized> SequenceConstraintTarget for $wrapper {
                    type Element = T::Element;
                }
                impl<T: VariableLengthSequenceTarget + ?Sized> VariableLengthSequenceTarget for $wrapper {}
                impl<T: UniqueItemsConstraintTarget + ?Sized> UniqueItemsConstraintTarget for $wrapper {}

                impl<T: MapConstraintTarget + ?Sized> MapConstraintTarget for $wrapper {
                    type Key = T::Key;
                    type Value = T::Value;
                }

                impl<T: TextConstraintTarget + ?Sized> TextConstraintTarget for $wrapper {}
                impl<T: DecimalConstraintTarget + ?Sized> DecimalConstraintTarget for $wrapper {}
                impl<T: TemporalConstraintTarget + ?Sized> TemporalConstraintTarget for $wrapper {}
            )+
        };
    }

    transparent_target!(Box<T>, std::rc::Rc<T>, std::sync::Arc<T>);

    /// Proves that a getter output can represent the value accepted by a
    /// setter for the same logical property.
    ///
    /// The implementation set preserves borrowing while recognizing the
    /// canonical owned forms supported by model properties.
    #[doc(hidden)]
    pub trait PropertyOutputCompatible<Setter: ?Sized> {}

    /// Lifetime-independent marker for a getter returning `&T`.
    #[doc(hidden)]
    pub struct BorrowedPropertyOutput<T: ?Sized>(PhantomData<fn() -> T>);

    /// Lifetime-independent marker for a getter returning `Option<&T>`.
    #[doc(hidden)]
    pub struct OptionalBorrowedPropertyOutput<T: ?Sized>(PhantomData<fn() -> T>);

    impl<T: ?Sized> PropertyOutputCompatible<T> for T {}
    impl<T: ?Sized> PropertyOutputCompatible<T> for BorrowedPropertyOutput<T> {}
    impl PropertyOutputCompatible<String> for BorrowedPropertyOutput<str> {}
    impl<T> PropertyOutputCompatible<Vec<T>> for BorrowedPropertyOutput<[T]> {}
    impl<T> PropertyOutputCompatible<Option<T>> for OptionalBorrowedPropertyOutput<T> {}
    impl PropertyOutputCompatible<Option<String>> for OptionalBorrowedPropertyOutput<str> {}

    /// Wraps a successfully validated merged property slice.
    #[doc(hidden)]
    #[must_use]
    pub const fn local_property_set(
        properties: &'static [crate::metadata::PropertyMetadata],
    ) -> crate::metadata::LocalPropertySet {
        crate::metadata::LocalPropertySet::new(properties)
    }

    /// Creates one generated field/getter/setter source fragment.
    #[doc(hidden)]
    #[must_use]
    pub const fn property_fragment(
        name: &'static str,
        type_expression: Option<&'static qubit_reflect::__private::codegen_v4::expression::TypeExpressionData>,
        source: crate::metadata::PropertyFragmentSource,
    ) -> crate::metadata::PropertyFragment {
        crate::metadata::PropertyFragment::new(name, type_expression, source)
    }

    /// Creates the generated metadata attached by one `ModelImpl` block.
    #[doc(hidden)]
    #[must_use]
    pub const fn model_impl_metadata(
        fragments: &'static [crate::metadata::PropertyFragment],
        properties: Result<&'static crate::metadata::LocalPropertySet, &'static crate::metadata::PropertyBuildErrors>,
    ) -> crate::metadata::ModelImplMetadata {
        crate::metadata::ModelImplMetadata::new(fragments, properties)
    }
}

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
    #[cfg(feature = "generic")]
    pub use qubit_reflect::capability::CapabilityRegistration;
    pub use qubit_reflect::descriptor::Visibility;

    pub use super::compile_assertions::BorrowedPropertyOutput;
    pub use super::compile_assertions::DecimalConstraintTarget;
    pub use super::compile_assertions::IdentifierType;
    pub use super::compile_assertions::MapConstraintTarget;
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
    use crate::metadata::TypeMetadata;
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

    #[doc(hidden)]
    #[must_use]
    pub struct GeneratedTypeMetadataBuilder {
        metadata: TypeMetadata,
    }

    /// Selects generated metadata when a nested field type has a model derive.
    #[doc(hidden)]
    pub trait NestedTypeMetadataProbe {
        /// Returns the generated metadata when the probed type has one.
        fn nested_type_metadata(self) -> Option<&'static TypeMetadata>;
    }

    impl<T: crate::metadata::HasTypeMetadata> NestedTypeMetadataProbe for &&::std::marker::PhantomData<T> {
        fn nested_type_metadata(self) -> Option<&'static TypeMetadata> {
            TypeMetadata::try_of::<T>().ok()
        }
    }

    impl<T> NestedTypeMetadataProbe for &::std::marker::PhantomData<T> {
        fn nested_type_metadata(self) -> Option<&'static TypeMetadata> {
            None
        }
    }

    impl GeneratedTypeMetadataBuilder {
        /// Starts building metadata for one reflected type.
        pub fn new<T: qubit_reflect::descriptor::Reflect + 'static>(
            model_id: Option<crate::metadata::ModelId>,
            fields: &'static [crate::metadata::FieldMetadata],
            role: &'static crate::metadata::RoleMetadata,
        ) -> Self {
            Self {
                metadata: TypeMetadata::new::<T>(model_id, fields, role),
            }
        }

        /// Adds generated property metadata to the builder.
        pub const fn properties(mut self, properties: &'static [crate::metadata::PropertyMetadata]) -> Self {
            self.metadata = self.metadata.with_properties(properties);
            self
        }

        /// Adds generated field property fragments to the builder.
        pub const fn property_fragments(mut self, fragments: &'static [crate::metadata::PropertyFragment]) -> Self {
            self.metadata = self.metadata.with_property_fragments(fragments);
            self
        }

        /// Adds lazy providers for concrete nested model field types.
        pub const fn nested_metadata_providers(
            mut self,
            providers: &'static [fn() -> Option<&'static crate::metadata::TypeMetadata>],
        ) -> Self {
            self.metadata = self.metadata.with_nested_metadata_providers(providers);
            self
        }

        /// Records the generic definition represented by this metadata.
        #[cfg(feature = "generic")]
        pub const fn generic_definition(mut self, definition: &'static crate::generic::GenericModelMetadata) -> Self {
            self.metadata = self.metadata.with_generic_definition(definition);
            self
        }

        /// Validates and finishes generated metadata for `T`.
        ///
        /// # Panics
        ///
        /// Panics when generated metadata does not match the reflected type.
        #[must_use]
        pub fn finish<T: 'static>(self) -> TypeMetadata {
            self.metadata.assert_valid_for::<T>();
            self.metadata
        }

        /// Finishes generated metadata for later fallible ABI validation.
        #[doc(hidden)]
        #[must_use]
        pub const fn finish_unchecked(self) -> TypeMetadata {
            self.metadata
        }
    }

    /// Builds one field metadata overlay from generated declarations.
    #[doc(hidden)]
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn field_metadata(
        owner: ::std::any::TypeId,
        variant: Option<usize>,
        index: usize,
        name: Option<&'static str>,
        visibility: qubit_reflect::descriptor::Visibility,
        expression: &'static qubit_reflect::__private::codegen_v4::expression::TypeExpressionData,
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
    #[doc(hidden)]
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn field_metadata_with_opaque_type_id(
        owner: ::std::any::TypeId,
        variant: Option<usize>,
        index: usize,
        name: Option<&'static str>,
        visibility: qubit_reflect::descriptor::Visibility,
        expression: &'static qubit_reflect::__private::codegen_v4::expression::TypeExpressionData,
        opaque_type_id: Option<::std::any::TypeId>,
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
    #[doc(hidden)]
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn generic_field_metadata(
        definition_path: &'static str,
        index: usize,
        variant: Option<usize>,
        name: Option<&'static str>,
        visibility: qubit_reflect::descriptor::Visibility,
        expression: &'static qubit_reflect::__private::codegen_v4::expression::TypeExpressionData,
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
    #[doc(hidden)]
    #[must_use]
    pub const fn property_metadata(
        name: &'static str,
        type_expression: Option<&'static qubit_reflect::__private::codegen_v4::expression::TypeExpressionData>,
        field: Option<&'static crate::metadata::FieldMetadata>,
        getter: Option<&'static crate::metadata::GetterMetadata>,
        setter: Option<&'static crate::metadata::SetterMetadata>,
    ) -> crate::metadata::PropertyMetadata {
        crate::metadata::PropertyMetadata::new(name, type_expression, field, getter, setter)
    }

    /// Builds entity-role metadata for an identifier field.
    #[doc(hidden)]
    #[must_use]
    pub const fn entity_role(
        identifier: &'static crate::metadata::FieldMetadata,
        identifier_accessor: crate::__private::IdentifierAccessor,
    ) -> crate::metadata::RoleMetadata {
        crate::metadata::RoleMetadata::Entity(crate::metadata::EntityMetadata::new(identifier, identifier_accessor))
    }

    /// Builds projection-role metadata and its optional source target.
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
    #[doc(hidden)]
    #[must_use]
    pub const fn model_role() -> crate::metadata::RoleMetadata {
        crate::metadata::RoleMetadata::Model(crate::metadata::ModelMetadata)
    }

    /// Builds value-role metadata.
    #[doc(hidden)]
    #[must_use]
    pub const fn value_role(
        transparent_field: Option<&'static crate::metadata::FieldMetadata>,
        canonical_codec: Option<&'static crate::metadata::CodecMetadata>,
    ) -> crate::metadata::RoleMetadata {
        crate::metadata::RoleMetadata::Value(crate::metadata::ValueMetadata::new(transparent_field, canonical_codec))
    }

    /// Builds metadata for one generated enum variant.
    #[doc(hidden)]
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn enum_variant_metadata(
        owner: ::std::any::TypeId,
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
    #[doc(hidden)]
    #[must_use]
    pub const fn enum_role(variants: &'static [crate::metadata::EnumVariantMetadata]) -> crate::metadata::RoleMetadata {
        crate::metadata::RoleMetadata::Enum(crate::metadata::EnumMetadata::new(variants))
    }

    /// Builds metadata for one generic model definition.
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
    #[doc(hidden)]
    #[must_use]
    pub fn leak<T: 'static>(value: T) -> &'static T {
        Box::leak(Box::new(value))
    }

    /// Leaks generated values as a static slice for metadata storage.
    #[doc(hidden)]
    #[must_use]
    pub fn leak_slice<T: 'static>(values: Vec<T>) -> &'static [T] {
        Box::leak(values.into_boxed_slice())
    }

    #[doc(hidden)]
    pub use crate::__qubit_model_with_generic_feature as with_generic_feature;
}
