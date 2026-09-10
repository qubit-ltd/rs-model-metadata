// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Options construction preserves defaults and exact field selection.

use std::num::NonZeroUsize;

use qubit_model_derive::Model;
use qubit_model_metadata::metadata::NavigationStep;
use qubit_model_metadata::metadata::ObjectPath;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::FieldPath;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationMode;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationOptionsBuilder;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_model_metadata::validation::ValidationSelection;
use qubit_reflect::ReflectedRef;
use qubit_validator::ValidatorRegistry;

#[Model]
struct SelectedFields {
    #[text(non_blank)]
    first: String,
    #[text(non_blank)]
    second: String,
}

#[Model]
struct SelectedEnvelope {
    fields: SelectedFields,
}

#[test]
fn object_path_contract_covers_current_parent_and_rendering() {
    let current = ObjectPath::current();
    assert!(current.steps().is_empty());
    assert!(!current.requires_parent());
    assert_eq!(current.to_string(), "");

    let path = ObjectPath::new(&[
        NavigationStep::Property("child"),
        NavigationStep::Parent,
        NavigationStep::Property("name"),
    ])
    .unwrap();
    assert_eq!(path.steps().len(), 3);
    assert!(!path.requires_parent());
    assert_eq!(path.to_string(), "child/../name");
    let external = ObjectPath::new(&[NavigationStep::Parent]).unwrap();
    assert!(external.requires_parent());
}

/// Owned segment input selects one complete nested path, without prefix
/// expansion.
#[test]
fn test_segment_selection_owns_names_and_matches_exact_nested_paths() {
    let root = TypeMetadata::of::<SelectedEnvelope>();
    let models = ModelRegistry::from_metadata(&[]).expect("isolated registry");
    let roots = [root, TypeMetadata::of::<SelectedFields>()];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("nested structure");
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("nested constraints");
    assert_eq!(plan.binding_count(), 2, "both nested declarations must be bound");
    let names = ["fields".to_owned(), "second".to_owned()];
    let selected = FieldPath::from_segments(names.iter());
    drop(names);
    assert_eq!(selected, FieldPath::new("fields.second"));
    let value = SelectedEnvelope {
        fields: SelectedFields {
            first: String::new(),
            second: String::new(),
        },
    };
    let all = plan
        .validate(ReflectedRef::new(&value), &ValidationOptions::default())
        .expect("unselected execution");
    assert_eq!(
        all.violations()
            .iter()
            .map(|violation| violation.path().render())
            .collect::<Vec<_>>(),
        ["fields.first", "fields.second"]
    );
    for (path, expected) in [
        (selected, vec!["fields.second"]),
        (FieldPath::from_segments(["fields"]), vec![]),
        (FieldPath::from_segments(["fields.second"]), vec![]),
    ] {
        let options = ValidationOptions::builder()
            .selection(ValidationSelection::Fields(vec![path]))
            .build();
        let report = plan
            .validate(ReflectedRef::new(&value), &options)
            .expect("selected execution");
        let actual: Vec<_> = report
            .violations()
            .iter()
            .map(|violation| violation.path().render())
            .collect();
        assert_eq!(actual, expected);
    }
}

/// Default construction paths agree without requiring compatibility setters.
#[test]
fn test_builder_preserves_default_options() {
    assert_eq!(ValidationOptions::builder().build(), ValidationOptions::default());
    assert_eq!(
        ValidationOptionsBuilder::default().build(),
        ValidationOptions::default()
    );
}

/// A completed builder selects the requested field before spending its budget.
#[test]
fn test_builder_field_selection_and_budget_execute_together() {
    let root = TypeMetadata::of::<SelectedFields>();
    let models = ModelRegistry::from_metadata(&[]).expect("isolated registry");
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("field structure");
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("standard constraints");
    let options = ValidationOptions::builder()
        .mode(ValidationMode::FailFast)
        .selection(ValidationSelection::Fields(vec![FieldPath::new("second")]))
        .max_depth(NonZeroUsize::new(1).expect("positive depth"))
        .max_nodes(NonZeroUsize::new(3).expect("root, field read and rule call"))
        .max_violations(NonZeroUsize::new(1).expect("one failure"))
        .max_comparisons(NonZeroUsize::new(1).expect("positive budget"))
        .build();
    let report = plan
        .validate(
            ReflectedRef::new(&SelectedFields {
                first: String::new(),
                second: String::new(),
            }),
            &options,
        )
        .expect("selected field fits the budget");
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.violations()[0].path().render(), "second");
    assert!(report.is_truncated());
}
