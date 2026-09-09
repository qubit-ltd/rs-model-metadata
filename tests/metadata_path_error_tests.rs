// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! A failed property path must not suppress independent relationship failures.

use qubit_model_metadata::__private::v6;
use qubit_model_metadata::metadata::DeclaredEntityTarget;
use qubit_model_metadata::metadata::FieldAttributeMetadata;
use qubit_model_metadata::metadata::FieldMetadata;
use qubit_model_metadata::metadata::FieldReferenceMetadata;
use qubit_model_metadata::metadata::ModelId;
use qubit_model_metadata::metadata::NavigationStep;
use qubit_model_metadata::metadata::ObjectPath;
use qubit_model_metadata::metadata::ReferenceSelection;
use qubit_model_metadata::metadata::SerdeFieldMetadata;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveErrorKind;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_reflect::Reflect;
use qubit_reflect::TypeDescriptor;
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
        let reference = v6::leak(FieldReferenceMetadata::new(
            v6::leak(DeclaredEntityTarget::ModelId(ModelId::new("missing.Target"))),
            v6::leak(ReferenceSelection::Entity),
            false,
            Some(v6::leak(
                ObjectPath::new(&[NavigationStep::Property("invalid"), NavigationStep::Property("value")]).unwrap(),
            )),
        ));
        let fields = v6::leak_slice(vec![
            FieldMetadata::from_reflect(descriptor.field_at(0).unwrap()),
            v6::field_metadata(
                descriptor.field_at(1).unwrap(),
                v6::leak_slice(vec![FieldAttributeMetadata::Reference(reference)]),
                &[],
                &[],
                &SerdeFieldMetadata::DEFAULT,
            ),
        ]);
        let properties = v6::leak_slice(
            fields
                .iter()
                .map(|field| v6::property_metadata(field.name().unwrap(), field.type_ref(), Some(field), None, None))
                .collect(),
        );
        v6::GeneratedTypeMetadataBuilder::new(
            descriptor,
            Some(ModelId::new("error.PathRoot")),
            fields,
            v6::leak(v6::model_role()),
        )
        .properties(properties)
        .finish::<Root>()
    })
}

v6::register_model_capability!(Root, metadata);

#[test]
fn test_path_conflict_and_missing_target_are_both_reported() {
    let models = ModelRegistry::try_global().unwrap();
    let errors = StructureResolver::new(ResolveInputs { roots: &[], models })
        .resolve()
        .unwrap_err();
    // Reachability validation also reports the invalid descriptor
    // independently.
    assert_eq!(errors.errors().len(), 3);
    let cause = errors
        .errors()
        .iter()
        .find(|error| error.kind() == ResolveErrorKind::MetadataResolution && error.path().is_some())
        .unwrap();
    assert_eq!(cause.model_id(), Some("error.PathRoot"));
    assert_eq!(cause.path().unwrap().to_string(), "target");
    assert_eq!(cause.object_path().unwrap().to_string(), "invalid/value");
    assert!(cause.cause().is_some());
    assert!(!cause.sources().is_empty());
    assert!(
        errors
            .errors()
            .iter()
            .any(|error| error.kind() == ResolveErrorKind::MissingModelId)
    );
    assert!(
        !errors
            .errors()
            .iter()
            .any(|error| error.kind() == ResolveErrorKind::MissingProperty)
    );
}
