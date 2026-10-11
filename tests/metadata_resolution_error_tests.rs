// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Metadata and property lookup distinguish invalid capabilities from absence.

use std::sync::OnceLock;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_model_metadata::__private::ModelMetadataProvider;
use qubit_model_metadata::__private::ModelTypeSeal;
use qubit_model_metadata::__private::TypeMetadataProvider as TypeMetadataProviderTrait;
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
use qubit_reflect::descriptor::StructKind;
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

static ALTERNATE_WRONG_DESCRIPTOR: TypeDescriptor = qubit_reflect::__private::codegen_v3::descriptor::struct_type::<
    Wrong<2>,
>("WrongAlternateShape", StructKind::Tuple, &[]);

static PANIC_ONCE_CALLS: AtomicUsize = AtomicUsize::new(0);

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata, capabilities(panic_once_capability))]
struct PanicOnce<const N: usize>;

#[allow(
    clippy::extra_unused_type_parameters,
    reason = "derive capability providers receive the concrete type parameter"
)]
fn panic_once_capability<T: 'static>() -> CapabilityDescriptor {
    CapabilityDescriptor::with_adapter(model_metadata_key(), panic_once_metadata as ModelMetadataProvider)
}

fn panic_once_metadata() -> &'static TypeMetadata {
    match PANIC_ONCE_CALLS.fetch_add(1, Ordering::SeqCst) {
        0 => panic_once_metadata_for_one(),
        1 => panic!("intentional provider panic on first lookup"),
        _ => panic_once_metadata_for_two(),
    }
}

fn panic_once_metadata_for_one() -> &'static TypeMetadata {
    static METADATA: OnceLock<TypeMetadata> = OnceLock::new();
    METADATA.get_or_init(|| {
        v7::GeneratedTypeMetadataBuilder::new(
            TypeDescriptor::of::<PanicOnce<1>>(),
            None,
            &[],
            v7::leak(v7::model_role()),
        )
        .finish::<PanicOnce<1>>()
    })
}

fn panic_once_metadata_for_two() -> &'static TypeMetadata {
    static METADATA: OnceLock<TypeMetadata> = OnceLock::new();
    METADATA.get_or_init(|| {
        v7::GeneratedTypeMetadataBuilder::new(
            TypeDescriptor::of::<PanicOnce<2>>(),
            None,
            &[],
            v7::leak(v7::model_role()),
        )
        .finish::<PanicOnce<2>>()
    })
}

impl ModelTypeSeal for PanicOnce<1> {}
impl TypeMetadataProviderTrait for PanicOnce<1> {
    fn __type_metadata() -> &'static TypeMetadata {
        panic_once_metadata_for_one()
    }
}

impl ModelTypeSeal for PanicOnce<2> {}
impl TypeMetadataProviderTrait for PanicOnce<2> {
    fn __type_metadata() -> &'static TypeMetadata {
        panic_once_metadata_for_two()
    }
}

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
fn test_metadata_for_caches_absence_and_structured_errors() {
    let reflection = RegistrySnapshotBuilder::new().build().unwrap();
    let models = ModelRegistry::from_reflect_registry(&reflection).unwrap();

    let missing = TypeDescriptor::of::<u8>();
    assert!(models.metadata_for(missing).unwrap().is_none());
    assert!(models.metadata_for(missing).unwrap().is_none());

    let conflict = TypeDescriptor::of::<Invalid<4>>();
    for _ in 0..2 {
        let error = models.metadata_for(conflict).unwrap_err();
        assert!(matches!(error, ModelMetadataError::Capability { .. }));
    }

    let wrong = TypeDescriptor::of::<Wrong<1>>();
    let first = models.metadata_for(wrong).unwrap_err();
    let second = models.metadata_for(wrong).unwrap_err();
    assert!(matches!(first, ModelMetadataError::Abi { .. }));
    assert!(matches!(second, ModelMetadataError::Abi { .. }));
    match (first, second) {
        (
            ModelMetadataError::Abi {
                type_id: first_id,
                source: first_source,
                ..
            },
            ModelMetadataError::Abi {
                type_id: second_id,
                source: second_source,
                ..
            },
        ) => {
            assert_eq!(first_id, second_id);
            assert_eq!(first_source.code(), second_source.code());
        }
        _ => unreachable!("both lookups preserve ABI errors"),
    }
}

#[test]
fn test_metadata_cache_separates_descriptors_with_the_same_type_id() {
    let descriptor = TypeDescriptor::of::<Wrong<2>>();
    let metadata = v7::leak(
        v7::GeneratedTypeMetadataBuilder::new(
            descriptor,
            Some(ModelId::new("error.WrongDescriptor")),
            &[],
            v7::leak(v7::model_role()),
        )
        .finish::<Wrong<2>>(),
    );
    let source = qubit_reflect::identity::FragmentIdentity::new("fixture", "alternate", 1, 1, "model", 1);
    let models = ModelRegistry::from_static_metadata(&[(metadata, &source)]).unwrap();
    assert!(models.metadata_for(descriptor).unwrap().is_some());

    assert_eq!(ALTERNATE_WRONG_DESCRIPTOR.type_id(), descriptor.type_id());
    assert_ne!(
        &ALTERNATE_WRONG_DESCRIPTOR as *const TypeDescriptor,
        descriptor as *const TypeDescriptor,
    );
    assert!(matches!(
        models.metadata_for(&ALTERNATE_WRONG_DESCRIPTOR),
        Err(ModelMetadataError::Abi { .. })
    ));
}

#[test]
fn test_metadata_provider_panic_does_not_initialize_cache_cell() {
    PANIC_ONCE_CALLS.store(0, Ordering::SeqCst);
    let descriptor = TypeDescriptor::of::<PanicOnce<1>>();
    let mut builder = RegistrySnapshotBuilder::new();
    builder.add_type(
        descriptor,
        qubit_reflect::identity::FragmentIdentity::new("fixture", "panic", 1, 1, "type", 1),
    );
    let reflection = builder.build().unwrap();
    let models = ModelRegistry::from_reflect_registry(&reflection).unwrap();

    let queried = TypeDescriptor::of::<PanicOnce<2>>();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| models.metadata_for(queried)));
    assert!(panic.is_err());
    let metadata = models
        .metadata_for(queried)
        .expect("retry after provider panic")
        .expect("provider metadata");
    assert!(std::ptr::eq(metadata, panic_once_metadata_for_two()));
    assert_eq!(PANIC_ONCE_CALLS.load(Ordering::SeqCst), 3);
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
