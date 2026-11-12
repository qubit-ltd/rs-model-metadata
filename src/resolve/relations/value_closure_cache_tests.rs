// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
// =============================================================================

use qubit_model_derive::Model;
use qubit_model_derive::Value;
use qubit_reflect::ReflectionContext;

use super::super::error::ResolveErrorKind;
use super::super::internal::ResolutionContext;
use super::super::internal::ValueClosureState;
use super::super::relations::validate_value_closure;
use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;
use crate::registry::ModelRegistryOrigin;

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

#[Model(eq, hash)]
struct InvalidModel {}

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
