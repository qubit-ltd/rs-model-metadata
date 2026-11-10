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
use qubit_model_metadata::metadata::PropertyBorrowed;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_validator::ValidationReport;
use qubit_validator::ValidatorRegistry;

#[Model]
struct Profile;

#[Model]
struct SharedValidatedLeaf {
    #[text(non_blank)]
    value: String,
}

#[Model]
struct SharedValidatedRoot {
    left: SharedValidatedLeaf,
    right: SharedValidatedLeaf,
}

#[Model]
struct UnrelatedValidatedModel {
    #[text(non_blank)]
    value: String,
}

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

    let contextual = plan
        .validate_with_context(&profile, &[], &ValidationOptions::default())
        .expect("typed validation with explicit ancestor context succeeds");
    assert!(contextual.is_valid());
    let dynamic = plan
        .validate_dynamic(PropertyBorrowed::new(&profile), &ValidationOptions::default())
        .expect("dynamic validation succeeds");
    assert!(dynamic.is_valid());
}

/// A shared child type retains one validation binding for each usage path.
#[test]
fn test_plan_keeps_occurrences_for_shared_nested_model_paths() {
    let root = TypeMetadata::of::<SharedValidatedRoot>();
    let models = ModelRegistry::try_global().expect("model registry");
    let graph = Arc::new(
        StructureResolver::for_static_roots(models, [root])
            .resolve()
            .expect("shared child graph resolves"),
    );
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph,
            validators: &validators,
        },
    )
    .expect("shared child declarations bind at both paths");
    assert_eq!(plan.binding_count(), 2);

    let value = SharedValidatedRoot {
        left: SharedValidatedLeaf { value: String::new() },
        right: SharedValidatedLeaf { value: String::new() },
    };
    let report = plan
        .validate(&value, &ValidationOptions::default())
        .expect("validation report is produced");
    let paths: Vec<_> = report
        .violations()
        .iter()
        .map(|violation| violation.path().render())
        .collect();
    assert_eq!(paths, ["left.value", "right.value"]);

    let value = SharedValidatedRoot {
        left: SharedValidatedLeaf {
            value: "valid".to_owned(),
        },
        right: SharedValidatedLeaf { value: String::new() },
    };
    let report = plan
        .validate(&value, &ValidationOptions::default())
        .expect("one invalid shared-child value is reported");
    let paths: Vec<_> = report
        .violations()
        .iter()
        .map(|violation| violation.path().render())
        .collect();
    assert_eq!(paths, ["right.value"]);
}

/// Adding an unrelated registered model does not change root plan bindings.
#[test]
fn test_shared_graph_matches_root_only_plan_and_validation() {
    let root = TypeMetadata::of::<SharedValidatedRoot>();
    let models = ModelRegistry::try_global().expect("model registry");
    let root_graph = Arc::new(
        StructureResolver::for_static_roots(models, [root])
            .resolve()
            .expect("root graph resolves"),
    );
    let shared_graph = Arc::new(
        StructureResolver::for_static_roots(
            models,
            [root, TypeMetadata::of::<UnrelatedValidatedModel>()],
        )
        .resolve()
        .expect("shared graph resolves"),
    );
    let validators = ValidatorRegistry::empty();
    let root_plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: root_graph,
            validators: &validators,
        },
    )
    .expect("root-only plan builds");
    let shared_plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: shared_graph,
            validators: &validators,
        },
    )
    .expect("shared graph plan builds");

    assert_eq!(root_plan.binding_count(), 2);
    assert_eq!(shared_plan.binding_count(), root_plan.binding_count());
    let value = SharedValidatedRoot {
        left: SharedValidatedLeaf {
            value: String::new(),
        },
        right: SharedValidatedLeaf {
            value: String::new(),
        },
    };
    let root_report = root_plan
        .validate(&value, &ValidationOptions::default())
        .expect("root-only validation report");
    let shared_report = shared_plan
        .validate(&value, &ValidationOptions::default())
        .expect("shared graph validation report");
    let paths = |report: &ValidationReport| {
        report
            .violations()
            .iter()
            .map(|violation| violation.path().render())
            .collect::<Vec<_>>()
    };
    assert_eq!(paths(&root_report), ["left.value", "right.value"]);
    assert_eq!(paths(&shared_report), paths(&root_report));
    assert_eq!(root_report.is_valid(), shared_report.is_valid());
    assert_eq!(root_report.skipped(), shared_report.skipped());
}
