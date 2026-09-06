// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! A failed property path must not suppress independent relationship failures.

use qubit_codec::ValueCodecRegistry;
use qubit_model_metadata::__private::v4;
use qubit_model_metadata::DeclaredEntityTarget;
use qubit_model_metadata::FieldAttributeMetadata;
use qubit_model_metadata::FieldMetadata;
use qubit_model_metadata::FieldReferenceMetadata;
use qubit_model_metadata::ModelId;
use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::ModelResolveErrorKind;
use qubit_model_metadata::ModelResolver;
use qubit_model_metadata::PropertyPath;
use qubit_model_metadata::ReferenceSelection;
use qubit_model_metadata::Reflect;
use qubit_model_metadata::ResolveInputs;
use qubit_model_metadata::SerdeFieldMetadata;
use qubit_model_metadata::TypeDescriptor;
use qubit_model_metadata::TypeMetadata;
use qubit_reflect::capability::CapabilityDescriptor;
use qubit_reflect::capability::CapabilityKey;
use qubit_reflect::identity::CapabilityId;

#[allow(
    clippy::extra_unused_type_parameters,
    reason = "derive capability providers receive the concrete type parameter"
)]
fn conflict<T: 'static>() -> CapabilityDescriptor {
    CapabilityDescriptor::with_adapter(CapabilityKey::new(CapabilityId::new("error.path").unwrap()), 1_usize)
}

fn second_conflict<T: 'static>() -> CapabilityDescriptor {
    conflict::<T>()
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata, capabilities(conflict, second_conflict))]
struct Invalid<const N: usize>;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct Root {
    invalid: Invalid<1>,
    target: u64,
}

fn metadata() -> &'static TypeMetadata {
    static METADATA: std::sync::OnceLock<TypeMetadata> = std::sync::OnceLock::new();
    METADATA.get_or_init(|| {
        let descriptor = TypeDescriptor::of::<Root>();
        let reference = v4::leak(FieldReferenceMetadata::new(
            v4::leak(DeclaredEntityTarget::ModelId(ModelId::new("missing.Target"))),
            v4::leak(ReferenceSelection::Entity),
            false,
            Some(v4::leak(PropertyPath::new(&["invalid", "value"]))),
        ));
        let fields = v4::leak_slice(vec![
            FieldMetadata::from_reflect(descriptor.field_at(0).unwrap()),
            v4::field_metadata(
                descriptor.field_at(1).unwrap(),
                v4::leak_slice(vec![FieldAttributeMetadata::Reference(reference)]),
                &[],
                &[],
                &SerdeFieldMetadata::DEFAULT,
            ),
        ]);
        let properties = v4::leak_slice(
            fields
                .iter()
                .map(|field| v4::property_metadata(field.name().unwrap(), field.type_ref(), Some(field), None, None))
                .collect(),
        );
        v4::GeneratedTypeMetadataBuilder::new(
            descriptor,
            Some(ModelId::new("error.PathRoot")),
            fields,
            v4::leak(v4::model_role()),
        )
        .properties(properties)
        .finish::<Root>()
    })
}

v4::register_model_capability!(Root, metadata);

#[test]
fn test_path_conflict_and_missing_target_are_both_reported() {
    let models = ModelRegistry::try_global().unwrap();
    let errors = ModelResolver::new(ResolveInputs {
        models,
        codecs: ValueCodecRegistry::global(),
    })
    .resolve_all()
    .unwrap_err();
    assert_eq!(errors.errors().len(), 2);
    let cause = errors
        .errors()
        .iter()
        .find(|error| error.kind() == ModelResolveErrorKind::MetadataResolution)
        .unwrap();
    assert_eq!(cause.model_id(), Some("error.PathRoot"));
    assert_eq!(cause.path().unwrap().to_string(), "invalid.value");
    assert!(cause.cause().is_some());
    assert!(!cause.sources().is_empty());
    assert!(
        errors
            .errors()
            .iter()
            .any(|error| error.kind() == ModelResolveErrorKind::MissingModelId)
    );
    assert!(
        !errors
            .errors()
            .iter()
            .any(|error| error.kind() == ModelResolveErrorKind::MissingProperty)
    );
}
