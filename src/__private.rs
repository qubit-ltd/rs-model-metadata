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

pub use inventory;
pub use qubit_codec;
pub use qubit_id;
pub use qubit_redact;
pub use qubit_reflect::__private::*;
pub use qubit_reflect::capability::TypeCapabilities as ReflectTypeCapabilities;
pub use qubit_reflect::capability::registered_type_capabilities;
pub use qubit_reflect::register_type_capabilities;
pub use qubit_validator;
pub use serde;

/// Serde predicates used by generated omission defaults.
#[doc(hidden)]
pub mod serde_helpers {
    /// Returns whether an optional generated value is absent.
    #[must_use]
    pub const fn is_none<T>(value: &Option<T>) -> bool {
        value.is_none()
    }

    pub trait IsEmpty {
        /// Returns whether this collection contains no values.
        fn is_empty(&self) -> bool;
    }

    macro_rules! impl_is_empty {
        ($($type:ty),+ $(,)?) => {
            $(impl<T> IsEmpty for $type {
                fn is_empty(&self) -> bool { self.is_empty() }
            })+
        };
    }

    impl_is_empty!(
        Vec<T>,
        std::collections::VecDeque<T>,
        std::collections::LinkedList<T>,
        std::collections::BinaryHeap<T>
    );

    impl<T: Ord> IsEmpty for std::collections::BTreeSet<T> {
        fn is_empty(&self) -> bool {
            self.is_empty()
        }
    }
    impl<T, S> IsEmpty for std::collections::HashSet<T, S> {
        fn is_empty(&self) -> bool {
            self.is_empty()
        }
    }
    impl<K: Ord, V> IsEmpty for std::collections::BTreeMap<K, V> {
        fn is_empty(&self) -> bool {
            self.is_empty()
        }
    }
    impl<K, V, S> IsEmpty for std::collections::HashMap<K, V, S> {
        fn is_empty(&self) -> bool {
            self.is_empty()
        }
    }

    /// Returns whether a generated collection contains no values.
    #[must_use]
    pub fn is_empty<T: IsEmpty>(value: &T) -> bool {
        value.is_empty()
    }
}

/// Marker implemented only by code generated from a model-role macro.
#[doc(hidden)]
pub trait ModelTypeSeal {}

/// Generated provider kept out of the public model bound.
#[doc(hidden)]
pub trait TypeMetadataProvider {
    fn __type_metadata() -> &'static crate::TypeMetadata;
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
    pub const fn local_property_set(properties: &'static [crate::PropertyMetadata]) -> crate::LocalPropertySet {
        crate::LocalPropertySet::new(properties)
    }

    /// Creates one generated field/getter/setter source fragment.
    #[doc(hidden)]
    #[must_use]
    pub const fn property_fragment(
        name: &'static str,
        type_ref: &'static TypeRef,
        source: crate::PropertyFragmentSource,
    ) -> crate::PropertyFragment {
        crate::PropertyFragment::new(name, type_ref, source)
    }

    /// Creates the generated metadata attached by one `ModelImpl` block.
    #[doc(hidden)]
    #[must_use]
    pub const fn model_impl_metadata(
        fragments: &'static [crate::PropertyFragment],
        properties: Result<&'static crate::LocalPropertySet, &'static crate::PropertyBuildErrors>,
    ) -> crate::ModelImplMetadata {
        crate::ModelImplMetadata::new(fragments, properties)
    }
}

/// Current generated-code ABI.
///
/// All intentionally permanent allocations used by generic metadata are
/// centralized here. Generated code must finish each aggregate through
/// [`v3::GeneratedTypeMetadataBuilder::finish`] so malformed metadata fails at
/// its construction boundary.
#[doc(hidden)]
pub mod v3 {
    pub use qubit_codec::ValueCodecDescriptor;
    pub use qubit_redact::Redact;
    pub use qubit_redact::Redactor;
    use qubit_reflect::FieldDescriptor;
    use qubit_reflect::VariantDescriptor;
    use qubit_reflect::descriptor::TypeRef;
    use qubit_reflect::expression::GenericDefinitionDescriptor;
    use qubit_reflect::identity::FragmentIdentity;

    pub use super::compile_assertions::*;
    use crate::TypeDescriptor;
    use crate::TypeMetadata;
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
            model_id: Option<crate::ModelId>,
            fields: &'static [crate::FieldMetadata],
            role: &'static crate::RoleMetadata,
        ) -> Self {
            Self {
                metadata: TypeMetadata::new(descriptor, model_id, fields, role),
            }
        }

        /// Adds generated property metadata to the builder.
        pub const fn properties(mut self, properties: &'static [crate::PropertyMetadata]) -> Self {
            self.metadata = self.metadata.with_properties(properties);
            self
        }

        /// Adds generated field property fragments to the builder.
        pub const fn property_fragments(mut self, fragments: &'static [crate::PropertyFragment]) -> Self {
            self.metadata = self.metadata.with_property_fragments(fragments);
            self
        }

        /// Records the generic definition represented by this metadata.
        pub const fn generic_definition(mut self, definition: &'static crate::GenericModelMetadata) -> Self {
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
        attributes: &'static [crate::FieldAttributeMetadata],
        constraints: &'static [crate::ConstraintMetadata],
        validators: &'static [crate::ValidatorMetadata],
        serde: &'static crate::SerdeFieldMetadata,
    ) -> crate::FieldMetadata {
        crate::FieldMetadata::with_semantics(reflect, attributes, constraints, validators, serde)
    }

    /// Builds one merged property metadata value.
    #[doc(hidden)]
    #[must_use]
    pub const fn property_metadata(
        name: &'static str,
        type_ref: &'static TypeRef,
        field: Option<&'static crate::FieldMetadata>,
        getter: Option<&'static crate::GetterMetadata>,
        setter: Option<&'static crate::SetterMetadata>,
    ) -> crate::PropertyMetadata {
        crate::PropertyMetadata::new(name, type_ref, field, getter, setter)
    }

    /// Builds entity-role metadata for an identifier field.
    #[doc(hidden)]
    #[must_use]
    pub const fn entity_role(identifier: &'static crate::FieldMetadata) -> crate::RoleMetadata {
        crate::RoleMetadata::Entity(crate::EntityMetadata::new(identifier))
    }

    /// Builds projection-role metadata and its optional source target.
    #[doc(hidden)]
    #[must_use]
    pub const fn projection_role(
        identifier: &'static crate::FieldMetadata,
        source: Option<&'static crate::DeclaredEntityTarget>,
    ) -> crate::RoleMetadata {
        crate::RoleMetadata::Projection(crate::ProjectionMetadata::new(identifier, source))
    }

    /// Builds metadata for a general model role.
    #[doc(hidden)]
    #[must_use]
    pub const fn model_role() -> crate::RoleMetadata {
        crate::RoleMetadata::Model(crate::ModelMetadata)
    }

    /// Builds value-role metadata.
    #[doc(hidden)]
    #[must_use]
    pub const fn value_role(
        transparent_field: Option<&'static crate::FieldMetadata>,
        canonical_codec: Option<&'static crate::CodecMetadata>,
    ) -> crate::RoleMetadata {
        crate::RoleMetadata::Value(crate::ValueMetadata::new(transparent_field, canonical_codec))
    }

    /// Builds metadata for one generated enum variant.
    #[doc(hidden)]
    #[must_use]
    pub const fn enum_variant_metadata(
        reflect: &'static VariantDescriptor,
        canonical_name: &'static str,
        serialized_name: &'static str,
        deserialized_name: &'static str,
        fields: &'static [crate::FieldMetadata],
        default: bool,
    ) -> crate::EnumVariantMetadata {
        crate::EnumVariantMetadata::new(
            reflect,
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
    pub const fn enum_role(variants: &'static [crate::EnumVariantMetadata]) -> crate::RoleMetadata {
        crate::RoleMetadata::Enum(crate::EnumMetadata::new(variants))
    }

    /// Builds metadata for one generic model definition.
    #[doc(hidden)]
    #[must_use]
    pub const fn generic_model_metadata(
        model_id: crate::ModelId,
        role: crate::ModelRole,
        definition: &'static GenericDefinitionDescriptor,
        fields: &'static [crate::FieldMetadata],
        variants: &'static [crate::EnumVariantMetadata],
    ) -> crate::GenericModelMetadata {
        crate::GenericModelMetadata::new(model_id, role, definition, fields, variants)
    }

    /// Builds a concrete generated model registration.
    #[doc(hidden)]
    #[must_use]
    pub const fn concrete_registration(
        metadata: &'static crate::TypeMetadata,
        source: &'static FragmentIdentity,
    ) -> crate::ModelRegistration {
        crate::ModelRegistration::from_concrete(metadata, source)
    }

    /// Builds a generic generated model registration.
    #[doc(hidden)]
    #[must_use]
    pub const fn generic_registration(
        metadata: &'static crate::GenericModelMetadata,
        source: &'static FragmentIdentity,
    ) -> crate::ModelRegistration {
        crate::ModelRegistration::from_generic(metadata, source)
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
    pub use crate::__qubit_model_register_model_capability as register_model_capability;
    pub use crate::__qubit_model_register_model_impl_capability as register_model_impl_capability;
}

/// Registers a generated property provider on the shared reflection root.
#[doc(hidden)]
#[macro_export]
macro_rules! __qubit_model_register_model_impl_capability {
    ($target:ty, $provider:expr $(,)?) => {
        $crate::__private::register_type_capabilities!(
            $target: [$crate::model_impl_key() => $provider]
        );
    };
}

/// Registers a model metadata provider as a typed reflection capability.
#[doc(hidden)]
#[macro_export]
macro_rules! __qubit_model_register_model_capability {
    ($target:ty, $provider:expr $(,)?) => {
        $crate::__private::register_type_capabilities!(
            $target: [$crate::model_metadata_key() => $provider]
        );
    };
}
