// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Regression tests for fallible property path traversal.

use qubit_codec::ValueCodecRegistry;
use qubit_reflect::__private::testing::build_registry;
use qubit_reflect::capability::CapabilityDescriptor;
use qubit_validator::ValidatorRegistry;

use super::ModelResolutionCause;
use super::resolve_property_path;
use crate::__private::v4;
use crate::ModelImplMetadata;
use crate::ModelRegistry;
use crate::PropertyBuildError;
use crate::PropertyBuildErrorKind;
use crate::PropertyBuildErrors;
use crate::PropertyPath;
use crate::PropertyResolutionError;
use crate::Reflect;
use crate::TypeDescriptor;

#[derive(Reflect)]
#[reflect(crate = crate, capabilities(broken_overlay))]
struct Broken<const N: usize>;

fn overlay() -> &'static ModelImplMetadata {
    static OVERLAY: std::sync::OnceLock<ModelImplMetadata> = std::sync::OnceLock::new();
    OVERLAY.get_or_init(|| {
        let error = PropertyBuildError::new(PropertyBuildErrorKind::GetterTypeMismatch, "value");
        ModelImplMetadata::new(&[], Err(v4::leak(PropertyBuildErrors::new(vec![error]))))
    })
}

#[allow(
    clippy::extra_unused_type_parameters,
    reason = "derive capability providers receive the concrete type parameter"
)]
fn broken_overlay<T: 'static>() -> CapabilityDescriptor {
    CapabilityDescriptor::with_adapter(crate::model_impl_key(), overlay as fn() -> &'static ModelImplMetadata)
}

#[test]
fn test_property_path_preserves_assembly_failure_instead_of_missing_property() {
    let reflection = build_registry(&[]).unwrap();
    let models = ModelRegistry::from_reflect_registry(&reflection).unwrap();
    let metadata = v4::leak(
        v4::GeneratedTypeMetadataBuilder::new(TypeDescriptor::of::<Broken<1>>(), None, &[], v4::leak(v4::model_role()))
            .finish::<Broken<1>>(),
    );
    let error = resolve_property_path(metadata, &PropertyPath::new(&["absent"]), &models).unwrap_err();
    let ModelResolutionCause::Properties(PropertyResolutionError::Assembly(errors)) = error else {
        panic!("expected original property assembly failure");
    };
    assert!(std::ptr::eq(errors, overlay().try_properties().unwrap_err()));
    assert_eq!(errors.errors()[0].property_name(), "value");
}

fn registered_metadata() -> &'static crate::TypeMetadata {
    static METADATA: std::sync::OnceLock<crate::TypeMetadata> = std::sync::OnceLock::new();
    METADATA.get_or_init(|| {
        v4::GeneratedTypeMetadataBuilder::new(
            TypeDescriptor::of::<Broken<2>>(),
            Some(crate::ModelId::new("error.Assembly")),
            &[],
            v4::leak(v4::model_role()),
        )
        .finish::<Broken<2>>()
    })
}

crate::register_reflected_type!(Broken<2>);
v4::register_model_capability!(Broken<2>, registered_metadata);

#[test]
fn test_assembly_diagnostics_keep_property_names_and_original_causes() {
    let models = ModelRegistry::try_global().unwrap();
    let errors = super::ModelResolver::new(super::ResolveInputs {
        models,
        validators: ValidatorRegistry::global(),
        codecs: ValueCodecRegistry::global(),
    })
    .resolve_all()
    .unwrap_err();
    let [error] = errors.errors() else {
        panic!("one assembly diagnostic")
    };
    assert_eq!(error.kind(), super::ModelResolveErrorKind::InvalidProperties);
    assert_eq!(error.model_id(), Some("error.Assembly"));
    assert_eq!(error.path().unwrap().to_string(), "value");
    assert!(std::error::Error::source(error).is_some());
    assert!(matches!(
        error.cause(),
        Some(ModelResolutionCause::Properties(PropertyResolutionError::Assembly(_)))
    ));
}
