// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Regression tests for fallible property path traversal.

use qubit_reflect::Reflect;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::capability::CapabilityDescriptor;
use qubit_reflect::register_reflected_type;
use qubit_reflect::registry::RegistrySnapshotBuilder;

use super::ModelResolutionCause;
use super::relations::resolve_property_path;
use crate::__private::v6;
use crate::metadata::ModelImplMetadata;
use crate::metadata::PropertyBuildError;
use crate::metadata::PropertyBuildErrorKind;
use crate::metadata::PropertyBuildErrors;
use crate::metadata::PropertyPath;
use crate::metadata::PropertyResolutionError;
use crate::registry::ModelRegistry;

#[derive(Reflect)]
#[reflect(crate = crate, capabilities(broken_overlay))]
struct Broken<const N: usize>;

fn overlay() -> &'static ModelImplMetadata {
    static OVERLAY: std::sync::OnceLock<ModelImplMetadata> = std::sync::OnceLock::new();
    OVERLAY.get_or_init(|| {
        let error = PropertyBuildError::new(PropertyBuildErrorKind::GetterTypeMismatch, "value");
        ModelImplMetadata::new(&[], Err(v6::leak(PropertyBuildErrors::new(vec![error]))))
    })
}

#[allow(
    clippy::extra_unused_type_parameters,
    reason = "derive capability providers receive the concrete type parameter"
)]
fn broken_overlay<T: 'static>() -> CapabilityDescriptor {
    CapabilityDescriptor::with_adapter(
        crate::reflect_facade::model_impl_key(),
        overlay as fn() -> &'static ModelImplMetadata,
    )
}

#[test]
fn test_property_path_preserves_assembly_failure_instead_of_missing_property() {
    let reflection = RegistrySnapshotBuilder::new().build().unwrap();
    let models = ModelRegistry::from_reflect_registry(&reflection).unwrap();
    let metadata = v6::leak(
        v6::GeneratedTypeMetadataBuilder::new(TypeDescriptor::of::<Broken<1>>(), None, &[], v6::leak(v6::model_role()))
            .finish::<Broken<1>>(),
    );
    let error = resolve_property_path(metadata, &PropertyPath::new(&["absent"]), &models).unwrap_err();
    let ModelResolutionCause::Properties(PropertyResolutionError::Assembly(errors)) = error else {
        panic!("expected original property assembly failure");
    };
    assert!(std::ptr::eq(errors, overlay().try_properties().unwrap_err()));
    assert_eq!(errors.errors()[0].property_name(), "value");
}

fn registered_metadata() -> &'static crate::metadata::TypeMetadata {
    static METADATA: std::sync::OnceLock<crate::metadata::TypeMetadata> = std::sync::OnceLock::new();
    METADATA.get_or_init(|| {
        v6::GeneratedTypeMetadataBuilder::new(
            TypeDescriptor::of::<Broken<2>>(),
            Some(crate::metadata::ModelId::new("error.Assembly")),
            &[],
            v6::leak(v6::model_role()),
        )
        .finish::<Broken<2>>()
    })
}

register_reflected_type!(Broken<2>);
v6::register_model_capability!(Broken<2>, registered_metadata);

#[test]
fn test_assembly_diagnostics_keep_property_names_and_original_causes() {
    let models = ModelRegistry::try_global().unwrap();
    let errors = super::StructureResolver::new(super::ResolveInputs { roots: &[], models })
        .resolve()
        .unwrap_err();
    let [error] = errors.errors() else {
        panic!("one assembly diagnostic")
    };
    assert_eq!(error.kind(), super::ResolveErrorKind::InvalidProperties);
    assert_eq!(error.model_id(), Some("error.Assembly"));
    assert_eq!(error.path().unwrap().to_string(), "value");
    assert!(std::error::Error::source(error).is_some());
    assert!(matches!(
        error.cause(),
        Some(ModelResolutionCause::Properties(PropertyResolutionError::Assembly(_)))
    ));
}
