// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![cfg(feature = "validation")]

//! Tests for explicit erased-selector execution capabilities.

use std::collections::HashMap;

use qubit_model_derive::Model;
use qubit_model_metadata::metadata::SelectorPosition;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildErrorKind;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationCapabilities;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::identity::FragmentIdentity;
use qubit_validator::ValidatorRegistry;

#[Model(id = "validation.UnsupportedMapKey", no_hash)]
struct UnsupportedMapKey {
    #[map_key(validator(id = "test.text"))]
    values: HashMap<String, String>,
}

#[test]
fn capability_matrix_is_explicit() {
    let models = ModelRegistry::try_global().unwrap();
    let graph = StructureResolver::new(ResolveInputs { models, roots: &[] })
        .resolve()
        .unwrap();
    let errors = ValidationCapabilities::check(TypeMetadata::of::<UnsupportedMapKey>(), &graph).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
    assert_eq!(errors[0].selector(), Some(SelectorPosition::MapKey));
}

#[test]
fn unsupported_map_selector_retains_model_path_and_position() {
    let source = FragmentIdentity::new("validation-tests", "fixture", line!(), 1, "model", 1);
    let metadata = TypeMetadata::of::<UnsupportedMapKey>();
    let models = ModelRegistry::from_metadata(&[(metadata, &source)]).expect("model registry");
    let graph = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: &models,
    })
    .resolve()
    .expect("model graph");
    let validators = ValidatorRegistry::empty();

    let result = ValidationPlan::build(
        metadata,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    );
    let Err(errors) = result else {
        panic!("map-key execution is unsupported")
    };
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
    assert_eq!(errors[0].model().unwrap().as_str(), "validation.UnsupportedMapKey");
    assert_eq!(errors[0].path(), Some("values"));
    assert_eq!(errors[0].selector(), Some(SelectorPosition::MapKey));
    assert!(errors[0].source_error().is_none());
}
