// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Validation plan ownership behavior.

use std::sync::Arc;

use qubit_model_derive::Model;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_validator::ValidatorRegistry;

#[Model]
struct Profile;

/// A plan keeps its resolved graph alive after the caller releases its handle.
#[test]
fn test_plan_retains_graph_after_caller_drops_arc() {
    let root: &'static TypeMetadata = TypeMetadata::of::<Profile>();
    let models = ModelRegistry::try_global().expect("model registry");
    let graph = Arc::new(
        StructureResolver::for_static_roots(models, [root])
            .resolve()
            .expect("valid structure"),
    );
    let validators = ValidatorRegistry::empty();
    let graph_ptr = Arc::as_ptr(&graph);
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: Arc::clone(&graph),
            validators: &validators,
        },
    )
    .expect("empty model plan builds");

    drop(graph);
    drop(validators);

    assert!(std::ptr::eq(plan.graph(), graph_ptr));
    let profile = Profile;
    let report = plan
        .validate(&profile, &ValidationOptions::default())
        .expect("validation succeeds");
    assert!(report.is_valid());
    assert!(report.violations().is_empty());
    assert!(report.skipped().is_empty());
}
