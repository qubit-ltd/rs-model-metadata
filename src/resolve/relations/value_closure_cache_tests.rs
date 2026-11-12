// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
// =============================================================================

use std::collections::BTreeMap;

use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::Value;
use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;

use super::super::error::ResolveErrorKind;
use super::super::internal::ResolutionContext;
use super::super::internal::ValueClosureState;
use super::super::relations::validate_value_closure;
use crate::metadata::FieldMetadata;
use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;
use crate::registry::ModelRegistryOrigin;
use crate::resolve::ResolveInputs;
use crate::resolve::StructureResolver;

static TEST_SOURCE: ModelRegistryOrigin = ModelRegistryOrigin {
    crate_name: "value-closure-cache-tests",
    crate_version: None,
    module_path: "resolve::relations::value_closure_cache_tests",
    file: Some(file!()),
    line: Some(1),
    column: Some(1),
};

#[Value]
struct SharedLeaf {
    text: String,
}

#[Value]
struct SharedBranch {
    left: SharedLeaf,
    right: SharedLeaf,
}

#[Value]
struct SharedRoot {
    left: SharedBranch,
    right: SharedBranch,
}

macro_rules! shared_value_level {
    ($name:ident, $child:ty) => {
        #[Value]
        struct $name {
            left: $child,
            right: $child,
        }
    };
}

shared_value_level!(SharedLevel1, SharedLeaf);
shared_value_level!(SharedLevel2, SharedLevel1);
shared_value_level!(SharedLevel3, SharedLevel2);
shared_value_level!(SharedLevel4, SharedLevel3);
shared_value_level!(SharedLevel5, SharedLevel4);
shared_value_level!(SharedLevel6, SharedLevel5);
shared_value_level!(SharedLevel7, SharedLevel6);
shared_value_level!(SharedLevel8, SharedLevel7);
shared_value_level!(SharedLevel9, SharedLevel8);
shared_value_level!(SharedLevel10, SharedLevel9);
shared_value_level!(SharedLevel11, SharedLevel10);
shared_value_level!(SharedLevel12, SharedLevel11);

#[Enum]
enum SharedChoice {
    Leaf(SharedLeaf),
    Branch(SharedBranch),
}

#[Value]
struct WrappedShared {
    nested: Option<Box<Vec<BTreeMap<String, (SharedLeaf, SharedChoice)>>>>,
}

#[Value]
struct WrappedSharedRoot {
    left: WrappedShared,
    right: WrappedShared,
}

#[Model(eq, hash)]
struct InvalidModel {}

#[Value]
struct OpaqueValue {
    visible: SharedLeaf,
    #[opaque]
    hidden: InvalidModel,
}

#[Value]
struct InvalidLeaf {
    model: InvalidModel,
}

#[Value]
struct InvalidSharedRoot {
    first: InvalidLeaf,
    second: InvalidLeaf,
}

#[Value]
struct CycleBefore {
    next: Option<Box<CycleBefore>>,
    invalid: InvalidModel,
}

#[Value]
struct CycleAfter {
    invalid: InvalidModel,
    next: Option<Box<CycleAfter>>,
}

#[test]
fn expands_a_shared_valid_closure_once_across_roots() {
    let root = TypeMetadata::of::<SharedRoot>();
    let branch = TypeMetadata::of::<SharedBranch>();
    let reflection = ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("linked reflection context");
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let roots = [root, branch];
    let resolution = ResolutionContext::new(&models, &roots);
    let mut state = ValueClosureState::new();
    let mut errors = Vec::new();

    validate_value_closure(root, &resolution, &mut state, None, &mut errors);
    validate_value_closure(branch, &resolution, &mut state, None, &mut errors);

    assert!(errors.is_empty());
    assert!(state.active_is_empty(), "each root releases active types");
    assert_eq!(
        state.expansion_count(),
        3,
        "root, branch, and leaf types only"
    );
}

#[test]
fn shared_dag_expansion_count_scales_with_unique_types_at_required_depths() {
    let reflection = ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("linked reflection context");
    let models = ModelRegistry::from_context(&reflection).expect("model registry");

    for (root, expected) in [
        (TypeMetadata::of::<SharedLevel1>(), 2),
        (TypeMetadata::of::<SharedLevel4>(), 5),
        (TypeMetadata::of::<SharedLevel8>(), 9),
        (TypeMetadata::of::<SharedLevel12>(), 13),
    ] {
        let resolution = ResolutionContext::new(&models, &[root]);
        let mut state = ValueClosureState::new();
        let mut errors = Vec::new();
        validate_value_closure(root, &resolution, &mut state, None, &mut errors);
        assert!(errors.is_empty());
        assert_eq!(state.expansion_count(), expected);
    }
}

#[test]
fn shared_closure_through_enum_and_supported_containers_is_valid() {
    let root = TypeMetadata::of::<WrappedSharedRoot>();
    let reflection = ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("linked reflection context");
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let resolution = ResolutionContext::new(&models, &[root]);
    let mut state = ValueClosureState::new();
    let mut errors = Vec::new();

    validate_value_closure(root, &resolution, &mut state, None, &mut errors);

    assert!(
        errors.is_empty(),
        "Option/Box/Vec/Map/tuple and Enum closure"
    );
    assert!(state.active_is_empty());
}

#[test]
fn invalid_shared_subtrees_preserve_both_diagnostic_paths_and_declarations() {
    let root = TypeMetadata::of::<InvalidSharedRoot>();
    let reflection = ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("linked reflection context");
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let resolution = ResolutionContext::new(&models, &[root]);
    let mut state = ValueClosureState::new();
    let mut errors = Vec::new();

    validate_value_closure(
        root,
        &resolution,
        &mut state,
        Some(&TEST_SOURCE),
        &mut errors,
    );

    assert_eq!(errors.len(), 2);
    assert!(
        errors
            .iter()
            .all(|error| error.kind() == ResolveErrorKind::InvalidValueClosure)
    );
    assert_eq!(
        errors
            .iter()
            .map(|error| error.path().unwrap().to_string())
            .collect::<Vec<_>>(),
        ["first.model", "second.model"]
    );
    assert!(
        errors
            .iter()
            .all(|error| error.owner_type_id() == Some(root.type_id()))
    );
    assert!(
        errors
            .iter()
            .all(|error| error.sources().len() == 1 && error.sources()[0] == TEST_SOURCE)
    );
    assert_eq!(
        errors
            .iter()
            .map(|error| error.declaration().unwrap().field)
            .collect::<Vec<_>>(),
        [Some(0), Some(0)]
    );
}

#[test]
fn self_cycle_before_or_after_invalid_sibling_keeps_the_sibling_diagnostic() {
    let before = TypeMetadata::of::<CycleBefore>();
    let after = TypeMetadata::of::<CycleAfter>();
    let reflection = ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("linked reflection context");
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    for (root, invalid_field) in [(before, 1), (after, 0)] {
        let resolution = ResolutionContext::new(&models, &[root]);
        let mut state = ValueClosureState::new();
        let mut errors = Vec::new();
        validate_value_closure(root, &resolution, &mut state, None, &mut errors);
        assert_eq!(errors.len(), 1);
        let error = &errors[0];
        assert_eq!(error.kind(), ResolveErrorKind::InvalidValueClosure);
        assert_eq!(error.path().unwrap().to_string(), "invalid");
        assert_eq!(error.owner_type_id(), Some(root.type_id()));
        assert_eq!(error.declaration().unwrap().field, Some(invalid_field));
    }
}

fn diagnostic_snapshot(root: &'static TypeMetadata, legacy: bool) -> Vec<String> {
    let reflection = ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("linked reflection context");
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let resolution = ResolutionContext::new(&models, &[root]);
    let mut state = if legacy {
        ValueClosureState::new_legacy_reference()
    } else {
        ValueClosureState::new()
    };
    let mut errors = Vec::new();
    validate_value_closure(
        root,
        &resolution,
        &mut state,
        Some(&TEST_SOURCE),
        &mut errors,
    );
    errors.iter().map(|error| format!("{error:?}")).collect()
}

#[test]
fn cached_diagnostics_match_uncached_legacy_reference_on_small_graphs() {
    for root in [
        TypeMetadata::of::<SharedRoot>(),
        TypeMetadata::of::<InvalidSharedRoot>(),
        TypeMetadata::of::<CycleBefore>(),
        TypeMetadata::of::<CycleAfter>(),
    ] {
        assert_eq!(
            diagnostic_snapshot(root, false),
            diagnostic_snapshot(root, true)
        );
    }
}

#[test]
fn opaque_fields_remain_outside_the_cached_value_closure() {
    let root = TypeMetadata::of::<OpaqueValue>();
    let reflection = ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("linked reflection context");
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let resolution = ResolutionContext::new(&models, &[root]);
    let mut state = ValueClosureState::new();
    let mut errors = Vec::new();

    validate_value_closure(root, &resolution, &mut state, None, &mut errors);

    assert!(errors.is_empty());
}

#[test]
fn fresh_resolve_uses_its_explicit_metadata_override_without_a_prior_cache_hit() {
    let reflection = ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("linked reflection context");
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let root = TypeMetadata::of::<SharedRoot>();
    let canonical_roots = [root];

    let _graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &canonical_roots,
    })
    .resolve()
    .expect("first resolver accepts the canonical Value metadata");

    let descriptor = TypeDescriptor::of::<SharedLeaf>();
    let fields = descriptor
        .fields()
        .iter()
        .map(|field| FieldMetadata::from_reflect(descriptor.type_id(), field))
        .collect();
    let overlay = crate::__private::v8::leak(
        crate::__private::v8::GeneratedTypeMetadataBuilder::new::<SharedLeaf>(
            None,
            crate::__private::v8::leak_slice(fields),
            crate::__private::v8::leak(crate::__private::v8::model_role()),
        )
        .finish::<SharedLeaf>(),
    );
    assert_eq!(
        overlay.type_id(),
        TypeMetadata::of::<SharedLeaf>().type_id()
    );
    let overlay_reflection = ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("second linked reflection context");
    let overlay_models =
        ModelRegistry::from_context(&overlay_reflection).expect("second model registry");
    let override_roots = [root, overlay];
    let errors = StructureResolver::new(ResolveInputs {
        models: &overlay_models,
        roots: &override_roots,
    })
    .resolve()
    .expect_err("fresh resolver must apply this attempt's explicit Model override");
    assert!(
        errors.errors().iter().any(|error| {
            error.kind() == ResolveErrorKind::InvalidValueClosure
                && error.actual_role() == Some(crate::metadata::ModelRole::Model)
        }),
        "explicit overlay diagnostics: {errors:?}"
    );
}
