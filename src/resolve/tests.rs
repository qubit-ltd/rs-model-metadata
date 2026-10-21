// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Regression tests for fallible property path traversal.
// This is a #[cfg(test)] child of resolve/mod.rs, with private path-resolution
// and property-assembly fixtures. Production type-to-file naming does not
// apply. qubit-style: allow type-file-name

use std::error::Error;
use std::sync::OnceLock;

use qubit_reflect::Reflect;
use qubit_reflect::ReflectionContext;

use super::ModelResolutionCause;
use super::ResolveErrorKind;
use super::ResolveInputs;
use super::StructureResolver;
use super::internal::ResolutionContext;
use super::relations::resolve_property_path;
use crate::__private::v8;
use crate::metadata::ModelId;
use crate::metadata::ModelImplMetadata;
use crate::metadata::PropertyBuildError;
use crate::metadata::PropertyBuildErrorKind;
use crate::metadata::PropertyBuildErrors;
use crate::metadata::PropertyPath;
use crate::metadata::PropertyResolutionError;
use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;
use crate::registry::ModelRegistryOrigin;

#[Reflect]
struct Broken<const N: usize>;

/// Constructs an invalid local assembly through the private constructor.
fn overlay() -> &'static ModelImplMetadata {
    static OVERLAY: OnceLock<ModelImplMetadata> = OnceLock::new();
    OVERLAY.get_or_init(|| {
        let error = PropertyBuildError::new(PropertyBuildErrorKind::GetterTypeMismatch, "value");
        ModelImplMetadata::new(&[], Err(v8::leak(PropertyBuildErrors::new(vec![error]))))
    })
}

/// Attaches the invalid assembly while leaving reflection itself well formed.
fn broken_overlay() -> &'static ModelImplMetadata {
    overlay()
}

crate::__qubit_model_submit_model_impl_fragment!(Broken<1>, broken_overlay, 1);
crate::__qubit_model_submit_model_impl_fragment!(Broken<2>, broken_overlay, 2);

#[test]
fn test_property_path_preserves_assembly_failure_instead_of_missing_property() {
    let reflection = ReflectionContext::builder()
        .include_type::<Broken<1>>()
        .register(crate::reflect_facade::model_impl_registration::<Broken<1>>())
        .build()
        .expect("reflection context should build");
    let models = ModelRegistry::from_context(&reflection).expect("model registry should build");
    let metadata = v8::leak(
        v8::GeneratedTypeMetadataBuilder::new::<Broken<1>>(None, &[], v8::leak(v8::model_role())).finish::<Broken<1>>(),
    );
    let context = models.context().expect("global registry context");
    let empty_merge = ModelImplMetadata::merge(metadata, &[], context);
    assert!(empty_merge.fragments().fragments().is_empty());
    assert!(
        empty_merge
            .try_properties()
            .expect("no providers produce an empty property view")
            .properties()
            .is_empty()
    );
    let context = ResolutionContext::new(&models, &[metadata]);
    let error = resolve_property_path(metadata, &PropertyPath::new(&["absent"]), &context).unwrap_err();
    let ModelResolutionCause::Properties(PropertyResolutionError::Assembly(errors)) = error else {
        panic!("expected original property assembly failure");
    };
    assert_eq!(
        errors.errors().len(),
        overlay().try_properties().unwrap_err().errors().len()
    );
    assert_eq!(errors.errors()[0].property_name(), "value");
}

/// Supplies the concrete model for the independently registered error fixture.
fn registered_metadata() -> &'static TypeMetadata {
    static METADATA: OnceLock<TypeMetadata> = OnceLock::new();
    METADATA.get_or_init(|| {
        v8::GeneratedTypeMetadataBuilder::new::<Broken<2>>(
            Some(ModelId::new("error.Assembly")),
            &[],
            v8::leak(v8::model_role()),
        )
        .finish::<Broken<2>>()
    })
}

#[test]
fn test_assembly_diagnostics_keep_property_names_and_original_causes() {
    let reflection = ReflectionContext::builder()
        .include_type::<Broken<2>>()
        .register(crate::reflect_facade::model_impl_registration::<Broken<2>>())
        .build()
        .expect("reflection context should build");
    static SOURCE: ModelRegistryOrigin = ModelRegistryOrigin {
        crate_name: "qubit-model-metadata",
        crate_version: None,
        module_path: "resolve::tests",
        file: Some(file!()),
        line: Some(line!()),
        column: None,
    };
    let models = ModelRegistry::from_static_metadata(&reflection, &[(registered_metadata(), &SOURCE)])
        .expect("static model registry should build");
    let errors = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: &models,
    })
    .resolve()
    .unwrap_err();
    let [error] = errors.errors() else {
        panic!("one assembly diagnostic")
    };
    assert_eq!(error.kind(), ResolveErrorKind::InvalidProperties);
    assert_eq!(error.model_id(), Some("error.Assembly"));
    assert_eq!(error.path().unwrap().to_string(), "value");
    assert!(Error::source(error).is_some());
    assert!(matches!(
        error.cause(),
        Some(ModelResolutionCause::Properties(PropertyResolutionError::Assembly(_)))
    ));
}

#[test]
fn test_static_root_resolution_keeps_root_after_caller_scope() {
    let reflection = ReflectionContext::builder()
        .include_type::<Broken<1>>()
        .register(crate::reflect_facade::model_impl_registration::<Broken<1>>())
        .build()
        .expect("reflection context should build");
    let models = ModelRegistry::from_context(&reflection).expect("model registry should build");
    let root = v8::leak(
        v8::GeneratedTypeMetadataBuilder::new::<Broken<1>>(None, &[], v8::leak(v8::model_role())).finish::<Broken<1>>(),
    );
    let resolver = StructureResolver::for_static_roots(&models, [root]);
    let errors = resolver.resolve().expect_err("broken property assembly is reported");
    assert!(errors.errors().iter().any(|error| {
        error.kind() == ResolveErrorKind::InvalidProperties
            && error.model_id().is_none()
            && error.path().is_some_and(|path| path.to_string() == "value")
    }));
}

#[test]
fn test_empty_property_path_does_not_resolve_a_property() {
    let reflection = ReflectionContext::builder()
        .include_type::<Broken<2>>()
        .register(crate::reflect_facade::model_impl_registration::<Broken<2>>())
        .build()
        .expect("reflection context should build");
    let models = ModelRegistry::from_context(&reflection).expect("model registry should build");
    let context = ResolutionContext::new(&models, &[]);
    let path = PropertyPath::new(&[]);

    assert!(
        resolve_property_path(registered_metadata(), &path, &context)
            .expect("empty path requires no property assembly")
            .is_none()
    );
}
