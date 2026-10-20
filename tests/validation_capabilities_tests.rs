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
use std::sync::Arc;

use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::registry::ModelRegistryOrigin;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildErrorKind;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationCapabilities;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::ReflectionContext;
use qubit_validator::ValidatorRegistry;

#[Model(id = "validation.MapKeySelector")]
struct MapKeySelector {
    #[map_key(text(non_blank))]
    values: HashMap<String, String>,
}

#[Model(id = "validation.OwnedSelectorGetter")]
struct OwnedSelectorGetter {
    #[element(text(non_blank))]
    values: Vec<String>,
}

#[Model(id = "validation.UnsupportedBoxedSliceSelector")]
struct UnsupportedBoxedSliceSelector {
    #[element(text(non_blank))]
    values: Box<[String]>,
}

#[ModelImpl]
impl OwnedSelectorGetter {
    pub fn values(&self) -> Vec<String> {
        self.values.clone()
    }
}

#[test]
fn test_capability_matrix_accepts_supported_map_key_selector() {
    let models = ModelRegistry::try_global().unwrap();
    let graph = StructureResolver::new(ResolveInputs { models, roots: &[] })
        .resolve()
        .unwrap();
    assert!(ValidationCapabilities::check(TypeMetadata::of::<MapKeySelector>(), &graph).is_ok());
}

#[test]
fn test_map_key_selector_violation_reports_map_key_path() {
    let source = ModelRegistryOrigin {
        crate_name: "validation-tests",
        crate_version: None,
        module_path: module_path!(),
        file: Some(file!()),
        line: Some(line!()),
        column: None,
    };
    let metadata = TypeMetadata::of::<MapKeySelector>();
    let models = ModelRegistry::from_static_metadata(ReflectionContext::global(), &[(metadata, &source)])
        .expect("model registry");
    let graph = Arc::new(
        StructureResolver::new(ResolveInputs {
            roots: &[],
            models: &models,
        })
        .resolve()
        .expect("model graph"),
    );
    let validators = ValidatorRegistry::empty();

    let result = ValidationPlan::build(
        metadata,
        ValidationBuildInputs {
            graph: Arc::clone(&graph),
            validators: &validators,
        },
    );
    let plan = result.expect("map-key selector plan builds");
    let report = plan
        .validate_dynamic(
            qubit_reflect::value::DynamicRef::<qubit_reflect::value::Local>::new(&MapKeySelector {
                values: HashMap::from([(String::new(), "value".to_owned())]),
            }),
            &qubit_model_metadata::validation::ValidationOptions::default(),
        )
        .expect("map-key validation executes");
    assert_eq!(report.violations().len(), 1);
    assert!(report.violations()[0].path().render().ends_with(".<map-key>"));
}

#[test]
fn test_owned_collection_getter_is_rejected_at_build() {
    let models = ModelRegistry::try_global().expect("model registry");
    let graph = StructureResolver::new(ResolveInputs { models, roots: &[] })
        .resolve()
        .expect("model graph");
    let errors = ValidationCapabilities::check(TypeMetadata::of::<OwnedSelectorGetter>(), &graph)
        .expect_err("owned collection output cannot provide member borrows");
    assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
}

#[test]
fn test_boxed_slice_selector_is_rejected_without_a_borrow_adapter() {
    let models = ModelRegistry::try_global().expect("model registry");
    let graph = StructureResolver::new(ResolveInputs { models, roots: &[] })
        .resolve()
        .expect("model graph");
    let errors = ValidationCapabilities::check(TypeMetadata::of::<UnsupportedBoxedSliceSelector>(), &graph)
        .expect_err("unsupported collection shapes remain build errors");
    assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
}
