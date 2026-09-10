// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Metadata and property lookup distinguish invalid capabilities from absence.

use qubit_model_metadata::__private::model_metadata_key;
use qubit_model_metadata::__private::v7;
use qubit_model_metadata::metadata::FieldMetadata;
use qubit_model_metadata::metadata::ModelId;
use qubit_model_metadata::metadata::ModelMetadataError;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ModelResolutionCause;
use qubit_model_metadata::resolve::ResolveErrorKind;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_reflect::Reflect;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::capability::CapabilityDescriptor;
use qubit_reflect::capability::CapabilityKey;
use qubit_reflect::identity::CapabilityId;
use qubit_reflect::registry::ReflectRegistry;
use qubit_reflect::registry::RegistrySnapshotBuilder;

/// The conflicting intrinsic contract used on unregistered const instances.
fn conflict_key() -> CapabilityKey<usize> {
    CapabilityKey::new(CapabilityId::new("example.metadata_conflict").unwrap())
}
/// First conflicting fact.
#[allow(
    clippy::extra_unused_type_parameters,
    reason = "derive capability providers receive the concrete type parameter"
)]
fn first<T: 'static>() -> CapabilityDescriptor {
    CapabilityDescriptor::with_adapter(conflict_key(), 1)
}
/// Second conflicting fact.
#[allow(
    clippy::extra_unused_type_parameters,
    reason = "derive capability providers receive the concrete type parameter"
)]
fn second<T: 'static>() -> CapabilityDescriptor {
    CapabilityDescriptor::with_adapter(conflict_key(), 2)
}
#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata, capabilities(first, second))]
struct Invalid<const N: usize>;

#[test]
fn test_metadata_and_properties_preserve_intrinsic_conflicts() {
    let reflection = RegistrySnapshotBuilder::new().build().unwrap();
    let models = ModelRegistry::from_reflect_registry(&reflection).unwrap();
    let descriptor = TypeDescriptor::of::<Invalid<1>>();
    assert!(models.metadata_for(descriptor).is_err());
    let metadata = v7::leak(
        v7::GeneratedTypeMetadataBuilder::new(descriptor, None, &[], v7::leak(v7::model_role())).finish::<Invalid<1>>(),
    );
    assert!(metadata.try_properties_in(&reflection).is_err());
    assert!(metadata.property_fragments_in(&reflection).is_err());
    assert!(metadata.try_property_in(&reflection, "absent").is_err());
    assert!(models.properties_for(metadata).is_err());
}

/// A value root with two independently invalid nested types.
#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct Root {
    first: Invalid<2>,
    second: Invalid<3>,
}

fn root_metadata() -> &'static TypeMetadata {
    static METADATA: std::sync::OnceLock<TypeMetadata> = std::sync::OnceLock::new();
    METADATA.get_or_init(|| {
        let descriptor = TypeDescriptor::of::<Root>();
        let fields = v7::leak_slice(
            descriptor
                .fields()
                .iter()
                .map(|field| FieldMetadata::from_reflect(field.declaring_type().type_id(), field))
                .collect(),
        );
        v7::GeneratedTypeMetadataBuilder::new(
            descriptor,
            Some(ModelId::new("error.Root")),
            fields,
            v7::leak(v7::value_role(None, None)),
        )
        .finish::<Root>()
    })
}

v7::register_model_capability!(Root, root_metadata);

#[allow(
    clippy::extra_unused_type_parameters,
    reason = "derive capability providers receive the concrete type parameter"
)]
fn wrong_provider<T: 'static>() -> CapabilityDescriptor {
    CapabilityDescriptor::with_adapter(model_metadata_key(), root_metadata as fn() -> &'static TypeMetadata)
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata, capabilities(wrong_provider))]
struct Wrong<const N: usize>;

#[test]
fn test_metadata_abi_failure_is_distinct_from_absence() {
    let reflection = RegistrySnapshotBuilder::new().build().unwrap();
    let models = ModelRegistry::from_reflect_registry(&reflection).unwrap();
    let error = models.metadata_for(TypeDescriptor::of::<Wrong<1>>()).unwrap_err();
    assert!(matches!(error, ModelMetadataError::Abi { .. }));
    assert!(std::error::Error::source(&error).is_some());
    assert!(models.metadata_for(TypeDescriptor::of::<u8>()).unwrap().is_none());
}

#[test]
fn test_resolver_aggregates_real_causes_without_false_role_errors() {
    let reflection = ReflectRegistry::initialize().unwrap();
    let models = ModelRegistry::from_reflect_registry(reflection).unwrap();
    let errors = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: &models,
    })
    .resolve()
    .unwrap_err();
    // Closure discovery and field traversal retain their distinct contexts.
    assert_eq!(errors.errors().len(), 4);
    let paths: Vec<_> = errors
        .errors()
        .iter()
        .map(|error| {
            assert_eq!(error.kind(), ResolveErrorKind::MetadataResolution);
            assert_eq!(error.model_id(), Some("error.Root"));
            assert!(!error.sources().is_empty());
            assert!(matches!(
                error.cause(),
                Some(ModelResolutionCause::Metadata(ModelMetadataError::Capability { .. }))
            ));
            error.path().map(|path| path.to_string())
        })
        .collect();
    assert_eq!(paths.iter().filter(|path| path.is_none()).count(), 2);
    assert_eq!(paths.into_iter().flatten().collect::<Vec<_>>(), ["first", "second"]);
}
