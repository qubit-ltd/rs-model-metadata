// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Hidden, versioned ABI consumed by generated model code.
// qubit-style: allow multiple-public-types
// qubit-style: allow type-file-name

pub use qubit_id;
pub use qubit_redact as redact;
pub use qubit_reflect::__private::codegen_v3;
pub use qubit_reflect::Reflect;
pub use qubit_reflect::ReflectedMut;
pub use qubit_reflect::ReflectedOwned;
pub use qubit_reflect::ReflectedRef;
pub use qubit_reflect::TypeDescriptor;
pub use qubit_reflect::capability::CapabilityDescriptor;
pub use qubit_reflect::capability::TypeCapabilities as ReflectTypeCapabilities;
pub use qubit_reflect::descriptor::TypeRef;
pub use qubit_reflect::expression::ConstExpression;
pub use qubit_reflect::expression::TypeExpression;
pub use qubit_reflect::reflect_impl;
pub use qubit_reflect::register_type_capabilities;
pub use serde;

pub use crate::metadata::NamedValidationArgument;
pub use crate::metadata::Sensitivity;
pub use crate::metadata::ValidationArgument;
pub use crate::reflect_facade::ModelImplProvider;
pub use crate::reflect_facade::ModelMetadataProvider;
pub use crate::reflect_facade::model_impl_fragment_key;
pub use crate::reflect_facade::model_impl_key;
pub use crate::reflect_facade::model_metadata_key;

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
    use qubit_reflect::descriptor::TypeRef;

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
        type_ref: &'static TypeRef,
        source: crate::metadata::PropertyFragmentSource,
    ) -> crate::metadata::PropertyFragment {
        crate::metadata::PropertyFragment::new(name, type_ref, source)
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

/// Current generated-code ABI.
///
/// All intentionally permanent allocations used by generic metadata are
/// centralized here. Generated code must finish each aggregate through
/// [`v6::GeneratedTypeMetadataBuilder::finish`] so malformed metadata fails at
/// its construction boundary.
#[doc(hidden)]
pub mod v6 {
    use qubit_reflect::FieldDefinitionDescriptor;
    use qubit_reflect::FieldDescriptor;
    #[cfg(feature = "generic")]
    use qubit_reflect::TypeDefinitionDescriptor;
    use qubit_reflect::TypeDescriptor;
    #[cfg(feature = "generic")]
    use qubit_reflect::VariantDefinitionDescriptor;
    use qubit_reflect::VariantDescriptor;
    use qubit_reflect::descriptor::TypeRef;

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
    pub use super::reflect_codegen::reflected_type_ref;
    pub use crate::metadata::RustTypeReference;
    use crate::metadata::TypeMetadata;
    #[cfg(feature = "generic")]
    pub use crate::reflect_facade::generic_model_capability;
    #[cfg(feature = "generic")]
    pub use crate::reflect_facade::generic_model_metadata_key;
    pub use crate::reflect_facade::model_capability;

    #[doc(hidden)]
    #[must_use]
    pub struct GeneratedTypeMetadataBuilder {
        metadata: TypeMetadata,
    }

    impl GeneratedTypeMetadataBuilder {
        /// Starts building metadata for one reflected type.
        pub const fn new(
            descriptor: &'static TypeDescriptor,
            model_id: Option<crate::metadata::ModelId>,
            fields: &'static [crate::metadata::FieldMetadata],
            role: &'static crate::metadata::RoleMetadata,
        ) -> Self {
            Self {
                metadata: TypeMetadata::new(descriptor, model_id, fields, role),
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
    pub const fn field_metadata(
        reflect: &'static FieldDescriptor,
        attributes: &'static [crate::metadata::FieldAttributeMetadata],
        constraints: &'static [crate::metadata::ConstraintMetadata],
        validators: &'static [crate::metadata::ValidatorMetadata],
        serde: &'static crate::metadata::SerdeFieldMetadata,
    ) -> crate::metadata::FieldMetadata {
        crate::metadata::FieldMetadata::with_semantics(reflect, attributes, constraints, validators, serde)
    }

    /// Builds a semantic overlay for one generic declaration field.
    #[doc(hidden)]
    #[must_use]
    pub fn generic_field_metadata(
        definition: &'static FieldDefinitionDescriptor,
        variant_inherited: bool,
        attributes: &'static [crate::metadata::FieldAttributeMetadata],
        constraints: &'static [crate::metadata::ConstraintMetadata],
        validators: &'static [crate::metadata::ValidatorMetadata],
        serde: &'static crate::metadata::SerdeFieldMetadata,
    ) -> crate::metadata::FieldMetadata {
        let symbolic_type = leak(TypeRef::Symbolic(definition.ty().clone()));
        crate::metadata::FieldMetadata::with_definition_semantics(
            definition,
            symbolic_type,
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
        type_ref: &'static TypeRef,
        field: Option<&'static crate::metadata::FieldMetadata>,
        getter: Option<&'static crate::metadata::GetterMetadata>,
        setter: Option<&'static crate::metadata::SetterMetadata>,
    ) -> crate::metadata::PropertyMetadata {
        crate::metadata::PropertyMetadata::new(name, type_ref, field, getter, setter)
    }

    /// Builds entity-role metadata for an identifier field.
    #[doc(hidden)]
    #[must_use]
    pub const fn entity_role(identifier: &'static crate::metadata::FieldMetadata) -> crate::metadata::RoleMetadata {
        crate::metadata::RoleMetadata::Entity(crate::metadata::EntityMetadata::new(identifier))
    }

    /// Builds projection-role metadata and its optional source target.
    #[doc(hidden)]
    #[must_use]
    pub const fn projection_role(
        identifier: &'static crate::metadata::FieldMetadata,
        source: Option<&'static crate::metadata::DeclaredEntityTarget>,
    ) -> crate::metadata::RoleMetadata {
        crate::metadata::RoleMetadata::Projection(crate::metadata::ProjectionMetadata::new(identifier, source))
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
    pub const fn enum_variant_metadata(
        reflect: &'static VariantDescriptor,
        canonical_name: &'static str,
        serialized_name: &'static str,
        deserialized_name: &'static str,
        fields: &'static [crate::metadata::FieldMetadata],
        default: bool,
    ) -> crate::metadata::EnumVariantMetadata {
        crate::metadata::EnumVariantMetadata::new(
            reflect,
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
    pub const fn generic_enum_variant_metadata(
        definition: &'static VariantDefinitionDescriptor,
        canonical_name: &'static str,
        serialized_name: &'static str,
        deserialized_name: &'static str,
        fields: &'static [crate::metadata::FieldMetadata],
        default: bool,
    ) -> crate::metadata::EnumVariantMetadata {
        crate::metadata::EnumVariantMetadata::from_definition(
            definition,
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
        definition: &'static TypeDefinitionDescriptor,
        fields: &'static [crate::metadata::FieldMetadata],
        variants: &'static [crate::metadata::EnumVariantMetadata],
    ) -> crate::generic::GenericModelMetadata {
        crate::generic::GenericModelMetadata::new(model_id.into(), role, definition, fields, variants)
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
    #[cfg(feature = "generic")]
    pub use crate::__qubit_model_register_generic_model_capability as register_generic_model_capability;
    #[doc(hidden)]
    pub use crate::__qubit_model_register_model_capability as register_model_capability;
    pub use crate::__qubit_model_register_model_impl_capability as register_model_impl_capability;
    pub use crate::__qubit_model_with_generic_feature as with_generic_feature;
}

/// Registers a generated property provider on the shared reflection root.
#[doc(hidden)]
#[macro_export]
macro_rules! __qubit_model_register_model_impl_capability {
    ($target:ty, $provider:expr, $fingerprint:expr $(,)?) => {
        const _: () = {
            fn runtime_identity() -> $crate::__private::codegen_v3::registration::RuntimeIdentity {
                $crate::__private::codegen_v3::registration::RuntimeIdentity::Capabilities(
                    $crate::__private::codegen_v3::registration::CapabilityTarget::Type(
                        ::core::any::TypeId::of::<$target>(),
                    ),
                )
            }
            fn payload() -> $crate::__private::codegen_v3::registration::FragmentPayload {
                $crate::__private::codegen_v3::registration::FragmentPayload::Capability(
                    $crate::__private::codegen_v3::registration::CapabilityRegistration::for_type(
                        $crate::__private::TypeDescriptor::of::<$target>(),
                        ::std::vec![$crate::__private::CapabilityDescriptor::with_adapter($crate::__private::model_impl_fragment_key(concat!("qubit.model.impl.v1.f", stringify!($fingerprint))), $provider)],
                    ),
                )
            }
            $crate::__private::codegen_v3::inventory::submit! {
                $crate::__private::codegen_v3::registration::RegistrationFragment::new(
                    $crate::__private::codegen_v3::registration::FragmentKind::Capability,
                    $crate::__private::codegen_v3::registration::StaticFragmentIdentity::new(
                        env!("CARGO_PKG_NAME"), module_path!(), line!(), column!(), concat!("model-impl<", stringify!($target), ">"), $fingerprint,
                    ),
                    runtime_identity,
                    payload,
                )
            }
        };
    };
    ($target:ty, $provider:expr $(,)?) => {
        $crate::__private::register_type_capabilities!(
            $target: [$crate::__private::model_impl_key() => $provider]
        );
    };
}

/// Registers a model metadata provider as a typed reflection capability.
#[doc(hidden)]
#[macro_export]
macro_rules! __qubit_model_register_model_capability {
    ($target:ty, $provider:expr $(,)?) => {
        $crate::__private::register_type_capabilities!(
            $target: [$crate::__private::model_metadata_key() => $provider]
        );
    };
}
