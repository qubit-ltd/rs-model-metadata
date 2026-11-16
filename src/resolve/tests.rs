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

use std::any::TypeId;
use std::sync::OnceLock;

use qubit_reflect::Reflect;
use qubit_reflect::ReflectionContext;

use super::ModelResolutionCause;
use super::internal::ResolutionContext;
use super::relations::reference_value_matches;
use super::relations::resolve_object_binding;
use super::relations::resolve_property_path;
use super::relations::validate_dependency;
use crate::__private::v8;
use crate::metadata::DependencyBindingMetadata;
use crate::metadata::ModelId;
use crate::metadata::ModelImplMetadata;
use crate::metadata::NavigationStep;
use crate::metadata::ObjectPath;
use crate::metadata::PropertyBuildError;
use crate::metadata::PropertyBuildErrorKind;
use crate::metadata::PropertyBuildErrors;
use crate::metadata::PropertyPath;
use crate::metadata::PropertyResolutionError;
use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;

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

#[test]
fn test_relation_helpers_preserve_root_escape_and_reference_type_rules() {
    let reflection = ReflectionContext::builder()
        .include_type::<Broken<2>>()
        .include_type::<Option<Broken<2>>>()
        .include_type::<String>()
        .register(crate::reflect_facade::model_impl_registration::<Broken<2>>())
        .build()
        .expect("reflection context should build");
    let models = ModelRegistry::from_context(&reflection).expect("model registry should build");
    let context = ResolutionContext::new(&models, &[registered_metadata()]);

    static ESCAPE_STEPS: [NavigationStep; 1] = [NavigationStep::Parent];
    let escape = ObjectPath::new(&ESCAPE_STEPS).expect("parent traversal is a valid declaration");
    let (resolved, escaped) = resolve_object_binding(registered_metadata(), &escape, &context)
        .expect("escaping the root is represented as a result");
    assert!(resolved.is_none());
    assert!(escaped);

    static DEPENDENCY_PROPERTY: [&str; 1] = ["value"];
    let deferred =
        DependencyBindingMetadata::new("value", PropertyPath::new(&DEPENDENCY_PROPERTY)).with_object_path(escape);
    let mut dependency_errors = Vec::new();
    validate_dependency(registered_metadata(), &deferred, &context, None, &mut dependency_errors);
    assert!(
        dependency_errors.is_empty(),
        "a dependency outside the root is deferred"
    );

    static MISSING_STEPS: [NavigationStep; 1] = [NavigationStep::Property("missing")];
    let missing = ObjectPath::new(&MISSING_STEPS).expect("property navigation is valid");
    let error = resolve_object_binding(registered_metadata(), &missing, &context)
        .expect_err("property assembly errors must be preserved during object navigation");
    let ModelResolutionCause::Properties(PropertyResolutionError::Assembly(errors)) = error else {
        panic!("expected original assembly failure");
    };
    assert_eq!(errors.errors()[0].property_name(), "value");

    let expected = context
        .reflection()
        .find_type_by_id(TypeId::of::<Broken<2>>())
        .expect("registered source descriptor")
        .expect("Broken has a reflection descriptor");
    let optional = context
        .reflection()
        .find_type_by_id(TypeId::of::<Option<Broken<2>>>())
        .expect("optional descriptor lookup")
        .expect("Option<Broken> has a reflection descriptor");
    let unrelated = context
        .reflection()
        .find_type_by_id(TypeId::of::<String>())
        .expect("String descriptor lookup")
        .expect("String has a reflection descriptor");
    assert!(reference_value_matches(expected, optional));
    assert!(!reference_value_matches(expected, unrelated));
}
