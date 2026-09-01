// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Hidden, versioned ABI consumed by generated model code.
// qubit-style: allow multiple-public-types
// qubit-style: allow type-file-name

pub use inventory;
pub use qubit_codec;
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
    pub const fn is_none<T>(value: &Option<T>) -> bool {
        value.is_none()
    }

    pub trait IsEmpty {
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

/// Marker implemented once by each generated `ModelProperties` block.
#[doc(hidden)]
pub trait ModelPropertiesSeal {}

/// Current generated-code ABI.
///
/// All intentionally permanent allocations used by generic metadata are
/// centralized here. Generated code must finish each aggregate through
/// [`v2::GeneratedTypeMetadataBuilder::finish`] so malformed metadata fails at
/// its construction boundary.
#[doc(hidden)]
pub mod v2 {
    use qubit_reflect::FieldDescriptor;
    use qubit_reflect::VariantDescriptor;
    use qubit_reflect::descriptor::TypeRef;
    use qubit_reflect::expression::GenericDefinitionDescriptor;
    use qubit_reflect::identity::FragmentIdentity;

    use crate::TypeDescriptor;
    use crate::TypeMetadata;
    pub use crate::reflect_facade::model_capability;

    #[doc(hidden)]
    #[must_use]
    pub struct GeneratedTypeMetadataBuilder {
        metadata: TypeMetadata,
    }

    impl GeneratedTypeMetadataBuilder {
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

        pub const fn properties(mut self, properties: &'static [crate::PropertyMetadata]) -> Self {
            self.metadata = self.metadata.with_properties(properties);
            self
        }

        pub const fn generic_definition(mut self, definition: &'static crate::GenericModelMetadata) -> Self {
            self.metadata = self.metadata.with_generic_definition(definition);
            self
        }

        #[must_use]
        pub fn finish<T: 'static>(self) -> TypeMetadata {
            self.metadata.assert_valid_for::<T>();
            self.metadata
        }
    }

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

    #[doc(hidden)]
    #[must_use]
    pub const fn entity_role(identifier: &'static crate::FieldMetadata) -> crate::RoleMetadata {
        crate::RoleMetadata::Entity(crate::EntityMetadata::new(identifier))
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn projection_role(
        identifier: &'static crate::FieldMetadata,
        source: Option<&'static crate::DeclaredEntityTarget>,
    ) -> crate::RoleMetadata {
        crate::RoleMetadata::Projection(crate::ProjectionMetadata::new(identifier, source))
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn model_role() -> crate::RoleMetadata {
        crate::RoleMetadata::Model(crate::ModelMetadata)
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn value_role(
        transparent_field: Option<&'static crate::FieldMetadata>,
        canonical_codec: Option<&'static crate::CodecMetadata>,
    ) -> crate::RoleMetadata {
        crate::RoleMetadata::Value(crate::ValueMetadata::new(transparent_field, canonical_codec))
    }

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

    #[doc(hidden)]
    #[must_use]
    pub const fn enum_role(variants: &'static [crate::EnumVariantMetadata]) -> crate::RoleMetadata {
        crate::RoleMetadata::Enum(crate::EnumMetadata::new(variants))
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn generic_model_metadata(
        model_id: crate::ModelId,
        role: crate::ModelRole,
        definition: &'static GenericDefinitionDescriptor,
        fields: &'static [crate::FieldMetadata],
    ) -> crate::GenericModelMetadata {
        crate::GenericModelMetadata::new(model_id, role, definition, fields)
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn concrete_registration(
        metadata: &'static crate::TypeMetadata,
        source: &'static FragmentIdentity,
    ) -> crate::ModelRegistration {
        crate::ModelRegistration::from_concrete(metadata, source)
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn generic_registration(
        metadata: &'static crate::GenericModelMetadata,
        source: &'static FragmentIdentity,
    ) -> crate::ModelRegistration {
        crate::ModelRegistration::from_generic(metadata, source)
    }

    #[doc(hidden)]
    #[must_use]
    pub fn leak<T: 'static>(value: T) -> &'static T {
        Box::leak(Box::new(value))
    }

    #[doc(hidden)]
    #[must_use]
    pub fn leak_slice<T: 'static>(values: Vec<T>) -> &'static [T] {
        Box::leak(values.into_boxed_slice())
    }

    #[doc(hidden)]
    pub use crate::__qubit_model_register_model_capability as register_model_capability;
    pub use crate::__qubit_model_register_properties_capability as register_properties_capability;
}

/// Registers a generated property provider on the shared reflection root.
#[doc(hidden)]
#[macro_export]
macro_rules! __qubit_model_register_properties_capability {
    ($target:ty, $provider:expr $(,)?) => {
        $crate::__private::register_type_capabilities!(
            $target: [$crate::model_properties_key() => $provider]
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
