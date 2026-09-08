// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Verifies model initialization preserves reflection registration failures.

use std::error::Error as _;

use qubit_model_metadata::__private::ModelImplProvider;
use qubit_model_metadata::__private::model_impl_key;
use qubit_model_metadata::__private::v5::register_model_impl_capability;
use qubit_model_metadata::metadata::ModelImplMetadata;
use qubit_model_metadata::metadata::PropertyResolutionError;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::registry::ModelRegistryErrorKind;
use qubit_reflect::Reflect;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::capability::CapabilityDescriptor;
use qubit_reflect::descriptor::TypeRef;
use qubit_reflect::identity::FragmentIdentity;
use qubit_reflect::register_reflected_type;
use qubit_reflect::registry::RegistrySnapshotBuilder;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct DuplicateReflectionSource;

register_reflected_type!(DuplicateReflectionSource);

#[test]
fn test_duplicate_concrete_source_is_reported_by_reflection_registry() {
    let error = ModelRegistry::try_global().expect_err(
        "duplicate reflection roots must invalidate model initialization",
    );

    assert_eq!(error.kind(), ModelRegistryErrorKind::ReflectionRegistry);
    assert_eq!(error.sources().len(), 2);
    assert!(
        error.source().is_some(),
        "model registry errors must preserve the reflection cause"
    );
    assert!(error.to_string().contains("reflection registry error"));
}

/// A global reflection error must not silently remove model implementation
/// properties.
#[test]
fn test_property_lookup_preserves_reflection_initialization_failure() {
    use qubit_model_metadata::__private::v5;
    let metadata = v5::leak(
        v5::GeneratedTypeMetadataBuilder::new(
            TypeDescriptor::of::<DuplicateReflectionSource>(),
            None,
            &[],
            v5::leak(v5::model_role()),
        )
        .finish::<DuplicateReflectionSource>(),
    );
    assert!(
        matches!(
            metadata.try_properties(),
            Err(PropertyResolutionError::Reflection(_))
        ),
        "registry failure must not become an empty property set"
    );
    assert!(metadata.property_fragments().is_err());
    let isolated = RegistrySnapshotBuilder::new()
        .build()
        .expect("empty isolated reflection");
    assert!(
        metadata
            .try_properties_in(&isolated)
            .expect("isolated properties")
            .properties()
            .is_empty()
    );
    assert!(
        metadata
            .property_fragments_in(&isolated)
            .expect("valid isolated capabilities")
            .is_empty()
    );
    let models =
        ModelRegistry::from_metadata(&[]).expect("isolated model registry");
    assert!(
        models
            .properties_for(metadata)
            .expect("isolated model properties")
            .properties()
            .is_empty()
    );
}

/// Supplies a method overlay distinguishable from the declaration's empty
/// properties.
fn overlay_provider() -> &'static ModelImplMetadata {
    use qubit_model_metadata::__private::v5;
    static OVERLAY: std::sync::OnceLock<ModelImplMetadata> =
        std::sync::OnceLock::new();
    OVERLAY.get_or_init(|| {
        let type_ref = v5::leak(TypeRef::Resolved(TypeDescriptor::of::<u32>()));
        let properties = v5::leak_slice(vec![v5::property_metadata(
            "computed", type_ref, None, None, None,
        )]);
        v5::model_impl_metadata(
            &[],
            Ok(v5::leak(v5::local_property_set(properties))),
        )
    })
}

register_model_impl_capability!(DuplicateReflectionSource, overlay_provider);

/// Explicit snapshots select their own overlay even when global initialization
/// fails.
#[test]
fn test_isolated_snapshot_selects_its_own_property_overlay() {
    use qubit_model_metadata::__private::v5;
    let mut snapshot = RegistrySnapshotBuilder::new();
    snapshot.add_type_capabilities(
        TypeDescriptor::of::<DuplicateReflectionSource>(),
        vec![CapabilityDescriptor::with_adapter(
            model_impl_key(),
            overlay_provider as ModelImplProvider,
        )],
        FragmentIdentity::new("model-test", "isolated", 1, 1, "capability", 1),
    );
    let reflection = snapshot.build().expect("isolated capability snapshot");
    let metadata = v5::leak(
        v5::GeneratedTypeMetadataBuilder::new(
            TypeDescriptor::of::<DuplicateReflectionSource>(),
            None,
            &[],
            v5::leak(v5::model_role()),
        )
        .finish::<DuplicateReflectionSource>(),
    );
    let models = ModelRegistry::from_reflect_registry(&reflection)
        .expect("isolated model projection");
    assert!(
        models
            .properties_for(metadata)
            .expect("overlay properties")
            .property("computed")
            .is_some()
    );
    let empty = RegistrySnapshotBuilder::new()
        .build()
        .expect("empty snapshot");
    assert!(
        metadata
            .try_property_in(&empty, "computed")
            .expect("no overlay")
            .is_none()
    );
    assert!(
        metadata.try_properties().is_err(),
        "global conflict remains visible"
    );
    assert!(
        metadata
            .try_property_in(&reflection, "computed")
            .expect("overlay still visible")
            .is_some()
    );
}
