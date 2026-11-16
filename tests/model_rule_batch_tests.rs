// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![cfg(feature = "validation")]

//! Model-rule batch construction and execution order.

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_model_derive::Model;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ModelRuleBinding;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::ReflectionContext;
use qubit_reflect::value::DynamicRef;
use qubit_reflect::value::Local;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidationContext;
use qubit_validator::DependencySpec;
use qubit_validator::ExecutionError;
use qubit_validator::InputType;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;

#[Model(id = "validation.ModelRuleBatch")]
struct BatchModel {
    value: String,
}

struct RejectModel;

impl PreparedValidator for RejectModel {
    fn input_type(&self) -> InputType {
        InputType::of::<BatchModel>()
    }

    fn dependency_specs(&self) -> &'static [DependencySpec] {
        &[]
    }

    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        assert!(value.typed::<BatchModel>().is_some());
        Ok(PreparedOutcome::Invalid(vec![ViolationDraft::new(ViolationCode::new(
            "model.invalid",
        ))]))
    }
}

struct RejectNumber {
    input_type_calls: Arc<AtomicUsize>,
    validate_calls: Arc<AtomicUsize>,
}

impl PreparedValidator for RejectNumber {
    fn input_type(&self) -> InputType {
        self.input_type_calls.fetch_add(1, Ordering::SeqCst);
        InputType::of::<u8>()
    }

    fn dependency_specs(&self) -> &'static [DependencySpec] {
        &[]
    }

    fn validate(
        &self,
        _: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        self.validate_calls.fetch_add(1, Ordering::SeqCst);
        Ok(PreparedOutcome::Valid)
    }
}

/// Creates a prepared model rule with a distinct report identity.
fn binding(id: &'static str) -> ModelRuleBinding {
    ModelRuleBinding::from_prepared::<BatchModel>(ValidatorId::new(id), Arc::new(RejectModel))
        .expect("prepared model shape")
}

/// Builds a plan and checks both binding count and ordered report entries.
fn assert_batches(batches: Vec<Vec<ModelRuleBinding>>, expected: &[&str]) {
    let metadata = TypeMetadata::of::<BatchModel>();
    let reflection = ReflectionContext::global();
    let models = ModelRegistry::from_context(reflection).expect("model registry");
    let graph = Arc::new(
        StructureResolver::new(ResolveInputs {
            roots: &[],
            models: &models,
        })
        .resolve()
        .expect("structure"),
    );
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        metadata,
        ValidationBuildInputs {
            graph: Arc::clone(&graph),
            validators: &validators,
        },
    )
    .expect("binding");
    let plan = batches.into_iter().fold(plan, |plan, bindings| {
        plan.with_model_rules(bindings).expect("model rules match plan root")
    });
    assert_eq!(plan.binding_count(), expected.len());
    let report = plan
        .validate_dynamic(
            DynamicRef::<Local>::new_any(&BatchModel {
                value: "value".to_owned(),
            }),
            &ValidationOptions::default(),
        )
        .expect("execution");
    let actual: Vec<_> = report
        .violations()
        .iter()
        .map(|violation| {
            assert_eq!(violation.path().render(), "");
            assert_eq!(violation.code().as_str(), "model.invalid");
            violation.rule_id().as_str()
        })
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(report.is_valid(), expected.is_empty());
}

#[test]
fn test_with_model_rules_empty_batch() {
    assert_batches(vec![Vec::new()], &[]);
}

#[test]
fn test_with_model_rules_single_binding() {
    assert_batches(vec![vec![binding("batch.one")]], &["batch.one"]);
}

#[test]
fn test_with_model_rules_preserves_input_order() {
    assert_batches(
        vec![vec![
            binding("batch.first"),
            binding("batch.second"),
            binding("batch.third"),
        ]],
        &["batch.first", "batch.second", "batch.third"],
    );
}

#[test]
fn test_with_model_rules_appends_after_existing_rules() {
    assert_batches(
        vec![
            vec![binding("batch.existing")],
            Vec::new(),
            vec![binding("batch.new.first"), binding("batch.new.second")],
        ],
        &["batch.existing", "batch.new.first", "batch.new.second"],
    );
}

#[test]
fn test_with_model_rules_preserves_duplicate_ids() {
    assert_batches(
        vec![vec![binding("batch.duplicate"), binding("batch.duplicate")]],
        &["batch.duplicate", "batch.duplicate"],
    );
}

#[test]
fn test_with_model_rules_rejects_mismatched_root_before_execution() {
    let metadata = TypeMetadata::of::<BatchModel>();
    let reflection = ReflectionContext::global();
    let models = ModelRegistry::from_context(reflection).expect("model registry");
    let graph = Arc::new(
        StructureResolver::new(ResolveInputs {
            roots: &[],
            models: &models,
        })
        .resolve()
        .expect("structure"),
    );
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        metadata,
        ValidationBuildInputs {
            graph,
            validators: &validators,
        },
    )
    .expect("binding");

    let input_type_calls = Arc::new(AtomicUsize::new(0));
    let validate_calls = Arc::new(AtomicUsize::new(0));
    let wrong = |id| {
        ModelRuleBinding::from_prepared::<u8>(
            ValidatorId::new(id),
            Arc::new(RejectNumber {
                input_type_calls: Arc::clone(&input_type_calls),
                validate_calls: Arc::clone(&validate_calls),
            }),
        )
        .expect("prepared validator accepts u8")
    };
    let first_wrong = wrong("batch.wrong.first");
    let second_wrong = wrong("batch.wrong.second");
    input_type_calls.store(0, Ordering::SeqCst);
    validate_calls.store(0, Ordering::SeqCst);

    let error = match plan.with_model_rules([binding("batch.match"), first_wrong, second_wrong]) {
        Ok(_) => panic!("wrong root must fail during append"),
        Err(error) => error,
    };

    assert_eq!(error.kind(), BindErrorKind::PreparedSignatureMismatch);
    assert_eq!(error.rule_id(), Some(ValidatorId::new("batch.wrong.first")));
    assert_eq!(input_type_calls.load(Ordering::SeqCst), 0);
    assert_eq!(validate_calls.load(Ordering::SeqCst), 0);
}
