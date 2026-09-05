// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Verifies model initialization preserves reflection registration failures.

use std::error::Error as _;

use qubit_model_metadata::__private::v4::register_model_impl_capability;
use qubit_model_metadata::ModelImplMetadata;
use qubit_model_metadata::ModelImplProvider;
use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::ModelRegistryErrorKind;
use qubit_model_metadata::PropertyResolutionError;
use qubit_model_metadata::Reflect;
use qubit_model_metadata::TypeDescriptor;
use qubit_model_metadata::TypeRef;
use qubit_model_metadata::model_impl_key;
use qubit_model_metadata::register_reflected_type;
use qubit_reflect::__private::testing::build_registry;
use qubit_reflect::capability::CapabilityDescriptor;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct DuplicateReflectionSource;

register_reflected_type!(DuplicateReflectionSource);

#[test]
fn test_duplicate_concrete_source_is_reported_by_reflection_registry() {
    let error =
        ModelRegistry::try_global().expect_err("duplicate reflection roots must invalidate model initialization");

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
    use qubit_model_metadata::__private::v4;
    let metadata = v4::leak(
        v4::GeneratedTypeMetadataBuilder::new(
            TypeDescriptor::of::<DuplicateReflectionSource>(),
            None,
            &[],
            v4::leak(v4::model_role()),
        )
        .finish::<DuplicateReflectionSource>(),
    );
    assert!(
        matches!(metadata.try_properties(), Err(PropertyResolutionError::Reflection(_))),
        "registry failure must not become an empty property set"
    );
    assert!(metadata.property_fragments().is_err());
    let isolated = build_registry(&[]).expect("empty isolated reflection");
    assert!(
        metadata
            .try_properties_in(&isolated)
            .expect("isolated properties")
            .properties()
            .is_empty()
    );
    assert!(metadata.property_fragments_in(&isolated).is_empty());
    let models = ModelRegistry::from_metadata(&[], &[]).expect("isolated model registry");
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
    use qubit_model_metadata::__private::v4;
    static OVERLAY: std::sync::OnceLock<ModelImplMetadata> = std::sync::OnceLock::new();
    OVERLAY.get_or_init(|| {
        let type_ref = v4::leak(TypeRef::Resolved(TypeDescriptor::of::<u32>()));
        let properties = v4::leak_slice(vec![v4::property_metadata("computed", type_ref, None, None, None)]);
        v4::model_impl_metadata(&[], Ok(v4::leak(v4::local_property_set(properties))))
    })
}

register_model_impl_capability!(DuplicateReflectionSource, overlay_provider);

/// Explicit snapshots select their own overlay even when global initialization
/// fails.
#[test]
fn test_isolated_snapshot_selects_its_own_property_overlay() {
    use qubit_model_metadata::__private::v4;
    use qubit_reflect::__private::codegen_v2::registration::CapabilityRegistration;
    use qubit_reflect::__private::codegen_v2::registration::CapabilityTarget;
    use qubit_reflect::__private::codegen_v2::registration::FragmentKind;
    use qubit_reflect::__private::codegen_v2::registration::FragmentPayload;
    use qubit_reflect::__private::codegen_v2::registration::RegistrationFragment;
    use qubit_reflect::__private::codegen_v2::registration::RuntimeIdentity;
    use qubit_reflect::__private::codegen_v2::registration::StaticFragmentIdentity;
    /// Returns the exact concrete target claimed by this isolated fragment.
    fn identity() -> RuntimeIdentity {
        RuntimeIdentity::Capabilities(CapabilityTarget::Type(
            std::any::TypeId::of::<DuplicateReflectionSource>(),
        ))
    }
    /// Supplies the same generated overlay without the conflicting type
    /// fragments.
    fn payload() -> FragmentPayload {
        FragmentPayload::Capability(CapabilityRegistration::for_type(
            TypeDescriptor::of::<DuplicateReflectionSource>(),
            vec![CapabilityDescriptor::with_adapter(
                model_impl_key(),
                overlay_provider as ModelImplProvider,
            )],
        ))
    }
    static FRAGMENT: RegistrationFragment = RegistrationFragment::new(
        FragmentKind::Capability,
        StaticFragmentIdentity::new("model-test", "isolated", 1, 1, "capability", 1),
        identity,
        payload,
    );
    let reflection = build_registry(&[&FRAGMENT]).expect("isolated capability snapshot");
    let metadata = v4::leak(
        v4::GeneratedTypeMetadataBuilder::new(
            TypeDescriptor::of::<DuplicateReflectionSource>(),
            None,
            &[],
            v4::leak(v4::model_role()),
        )
        .finish::<DuplicateReflectionSource>(),
    );
    let models = ModelRegistry::from_reflect_registry(&reflection).expect("isolated model projection");
    assert!(
        models
            .properties_for(metadata)
            .expect("overlay properties")
            .property("computed")
            .is_some()
    );
    let empty = build_registry(&[]).expect("empty snapshot");
    assert!(
        metadata
            .try_property_in(&empty, "computed")
            .expect("no overlay")
            .is_none()
    );
    assert!(metadata.try_properties().is_err(), "global conflict remains visible");
    assert!(
        metadata
            .try_property_in(&reflection, "computed")
            .expect("overlay still visible")
            .is_some()
    );
}
