// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Options construction preserves defaults and exact field selection.

use std::cell::Cell;
use std::num::NonZeroUsize;
use std::sync::Arc;

use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
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
use qubit_validator::BoundValidationContext;
use qubit_validator::DependencySpec;
use qubit_validator::ExecutionError;
use qubit_validator::InputType;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistry;

thread_local! {
    static SELECTION_GETTER_CALLS: Cell<usize> = const { Cell::new(0) };
}

#[Model]
struct SelectedFields {
    #[text(non_blank)]
    first: String,
    #[text(non_blank)]
    second: String,
    plain: String,
}

#[ModelImpl]
impl SelectedFields {
    pub fn first(&self) -> &str {
        SELECTION_GETTER_CALLS.with(|calls| calls.set(calls.get() + 1));
        &self.first
    }
}

#[Model]
struct SelectedEnvelope {
    fields: SelectedFields,
}

struct AlwaysValid;

impl PreparedValidator for AlwaysValid {
    fn input_type(&self) -> InputType {
        InputType::of::<SelectedFields>()
    }

    fn dependency_specs(&self) -> &'static [DependencySpec] {
        &[]
    }

    fn validate(
        &self,
        _: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        Ok(PreparedOutcome::Valid)
    }
}

#[test]
fn test_object_path_contract_covers_current_parent_and_rendering() {
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
    let models = ModelRegistry::from_static_metadata(&[]).expect("isolated registry");
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
            plain: String::new(),
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
    let explicitly_all = plan
        .validate(
            ReflectedRef::new(&value),
            &ValidationOptions::builder().selection(ValidationSelection::All).build(),
        )
        .expect("All selection validates every bound occurrence");
    assert_eq!(explicitly_all.violations().len(), 2);
    let options = ValidationOptions::builder()
        .selection(ValidationSelection::Fields(vec![selected.clone(), selected]))
        .build();
    let report = plan
        .validate(ReflectedRef::new(&value), &options)
        .expect("repeated selection executes one matching occurrence");
    let actual: Vec<_> = report
        .violations()
        .iter()
        .map(|violation| violation.path().render())
        .collect();
    assert_eq!(actual, ["fields.second"]);

    for unmatched in [
        FieldPath::from_segments(["fields"]),
        FieldPath::from_segments(["fields.second"]),
    ] {
        let options = ValidationOptions::builder()
            .selection(ValidationSelection::Fields(vec![unmatched.clone()]))
            .build();
        let error = plan
            .validate(ReflectedRef::new(&value), &options)
            .expect_err("non-matching path must be rejected");
        assert_eq!(
            error.error().kind(),
            qubit_validator::ExecutionErrorKind::InvalidSelection
        );
        let retained_path = error
            .error()
            .trusted_source()
            .and_then(|source| source.downcast_ref::<FieldPath>())
            .expect("the rejected field path is retained as the trusted source");
        assert_eq!(retained_path, &unmatched);
        assert_eq!(
            retained_path.segments(),
            unmatched.segments(),
            "trusted callers can inspect the original segment sequence"
        );
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

/// Every requested field must correspond to a bound rule before validation
/// starts; rejected selections must not invoke any getter.
#[test]
fn test_field_selection_rejects_paths_without_bound_rules_before_execution() {
    let root = TypeMetadata::of::<SelectedFields>();
    let models = ModelRegistry::from_static_metadata(&[]).expect("isolated registry");
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
    let value = SelectedFields {
        first: String::new(),
        second: String::new(),
        plain: String::new(),
    };

    for (selection, expected_path) in [
        (vec![FieldPath::new("typo")], FieldPath::new("typo")),
        (
            vec![FieldPath::new("first"), FieldPath::new("typo")],
            FieldPath::new("typo"),
        ),
        (vec![FieldPath::new("plain")], FieldPath::new("plain")),
        (vec![], FieldPath::from_segments(std::iter::empty::<String>())),
        (
            vec![FieldPath::from_segments(std::iter::empty::<String>())],
            FieldPath::from_segments(std::iter::empty::<String>()),
        ),
    ] {
        SELECTION_GETTER_CALLS.with(|calls| calls.set(0));
        let options = ValidationOptions::builder()
            .selection(ValidationSelection::Fields(selection))
            .build();
        let error = plan
            .validate(ReflectedRef::new(&value), &options)
            .expect_err("unmatched selection must be rejected");
        assert_eq!(
            error.error().kind(),
            qubit_validator::ExecutionErrorKind::InvalidSelection
        );
        assert!(error.error().path().as_segments().is_empty());
        let retained_path = error
            .error()
            .trusted_source()
            .and_then(|source| source.downcast_ref::<FieldPath>())
            .expect("the first unmatched owned field path must be retained");
        assert_eq!(retained_path, &expected_path);
        assert_eq!(
            retained_path.segments(),
            expected_path.segments(),
            "trusted callers can inspect the exact unmatched segments"
        );
        assert!(error.partial_report().violations().is_empty());
        assert!(error.partial_report().skipped().is_empty());
        assert!(!error.partial_report().is_truncated());
        assert_eq!(error.root_type_id(), Some(root.type_id()));
        assert_eq!(
            SELECTION_GETTER_CALLS.with(Cell::get),
            0,
            "selection preflight must run before getters"
        );
    }
}

#[test]
fn test_explicit_empty_path_selects_model_level_rule() {
    let root = TypeMetadata::of::<SelectedFields>();
    let models = ModelRegistry::from_static_metadata(&[]).expect("isolated registry");
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
    .expect("standard constraints")
    .with_model_rules([
        qubit_model_metadata::validation::ModelRuleBinding::from_prepared::<SelectedFields>(
            ValidatorId::new("selection.always_valid"),
            Arc::new(AlwaysValid),
        )
        .expect("prepared model rule shape"),
    ]);
    let options = ValidationOptions::builder()
        .selection(ValidationSelection::Fields(vec![FieldPath::from_segments(
            std::iter::empty::<String>(),
        )]))
        .build();
    let report = plan
        .validate(
            ReflectedRef::new(&SelectedFields {
                first: String::new(),
                second: String::new(),
                plain: String::new(),
            }),
            &options,
        )
        .expect("explicit empty path selects the model-level occurrence");
    assert!(report.violations().is_empty());
}

/// A completed builder selects the requested field before spending its budget.
#[test]
fn test_builder_field_selection_and_budget_execute_together() {
    let root = TypeMetadata::of::<SelectedFields>();
    let models = ModelRegistry::from_static_metadata(&[]).expect("isolated registry");
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
                plain: String::new(),
            }),
            &options,
        )
        .expect("selected field fits the budget");
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.violations()[0].path().render(), "second");
    assert!(!report.is_truncated());
}
