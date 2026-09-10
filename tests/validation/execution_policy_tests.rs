// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Global stopping policies must bound reports and actual work.

use std::any::TypeId;
use std::cell::Cell;
use std::error::Error;
use std::num::NonZeroUsize;
use std::sync::Arc;

use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ModelRuleBinding;
use qubit_model_metadata::validation::ModelValidationError;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationMode;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::ReflectedRef;
use qubit_validator::BindError;
use qubit_validator::BoundValidationContext;
use qubit_validator::DependencySpec;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::RuleOutcome;
use qubit_validator::SkipReason;
use qubit_validator::ValidationReport;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorSignature;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;

thread_local! {
    static GETTER_CALLS: Cell<usize> = const { Cell::new(0) };
}

#[Model(id = "execution.Root")]
struct Root {
    #[text(non_blank)]
    value: String,
}
#[ModelImpl]
impl Root {
    pub fn value(&self) -> &str {
        GETTER_CALLS.with(|calls| calls.set(calls.get() + 1));
        &self.value
    }
}

struct Many {
    prerequisite: bool,
}
impl PreparedValidator for Many {
    fn validate(&self, _: ValidationValue<'_>, _: &BoundValidationContext<'_>) -> Result<RuleOutcome, ExecutionError> {
        let violations = (0..3)
            .map(|_| Violation::new(ValidatorId::new("execution.many"), ViolationCode::new("bad")))
            .collect();
        Ok(if self.prerequisite {
            RuleOutcome::Skipped {
                reason: SkipReason::FailedPrerequisite,
                prerequisites: violations,
            }
        } else {
            RuleOutcome::Invalid(violations)
        })
    }
}

/// Runs a model rule before a field whose getter records whether it was read.
fn assert_stopped(prerequisite: bool, options: ValidationOptions, expected: usize) {
    GETTER_CALLS.with(|calls| calls.set(0));
    let models = ModelRegistry::try_global().unwrap();
    let graph = StructureResolver::new(ResolveInputs { models, roots: &[] })
        .resolve()
        .unwrap();
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        TypeMetadata::of::<Root>(),
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .unwrap()
    .with_model_rule(ModelRuleBinding::from_prepared::<Root>(
        ValidatorId::new("execution.many"),
        Arc::new(Many { prerequisite }),
    ));
    let report = plan
        .validate(ReflectedRef::new(&Root { value: String::new() }), &options)
        .unwrap();
    assert_eq!(report.violations().len(), expected, "report must obey the hard limit");
    assert_eq!(
        GETTER_CALLS.with(Cell::get),
        0,
        "stopped plans must not execute subsequent getters"
    );
    assert!(report.is_truncated());
}

#[test]
fn test_fail_fast_retains_only_one_violation_and_stops_before_fields() {
    assert_stopped(
        false,
        ValidationOptions::builder().mode(ValidationMode::FailFast).build(),
        1,
    );
}

#[test]
fn test_violation_limit_stops_before_fields() {
    assert_stopped(
        false,
        ValidationOptions::builder()
            .max_violations(NonZeroUsize::new(1).unwrap())
            .build(),
        1,
    );
}

#[test]
fn test_prerequisites_share_the_same_hard_report_limit() {
    assert_stopped(
        true,
        ValidationOptions::builder()
            .max_violations(NonZeroUsize::new(1).unwrap())
            .build(),
        1,
    );
}

struct InvalidSkip {
    missing: bool,
}
impl PreparedValidator for InvalidSkip {
    fn validate(&self, _: ValidationValue<'_>, _: &BoundValidationContext<'_>) -> Result<RuleOutcome, ExecutionError> {
        Ok(if self.missing {
            RuleOutcome::Skipped {
                reason: SkipReason::MissingOptional,
                prerequisites: vec![Violation::new(
                    ValidatorId::new("execution.skip"),
                    ViolationCode::new("bad"),
                )],
            }
        } else {
            RuleOutcome::Skipped {
                reason: SkipReason::FailedPrerequisite,
                prerequisites: vec![],
            }
        })
    }
}

#[test]
fn test_invalid_skip_contracts_are_execution_errors() {
    let models = ModelRegistry::try_global().unwrap();
    let graph = StructureResolver::new(ResolveInputs { models, roots: &[] })
        .resolve()
        .unwrap();
    let validators = ValidatorRegistry::empty();
    for missing in [false, true] {
        let plan = ValidationPlan::build(
            TypeMetadata::of::<Root>(),
            ValidationBuildInputs {
                graph: &graph,
                validators: &validators,
            },
        )
        .unwrap()
        .with_model_rule(ModelRuleBinding::from_prepared::<Root>(
            ValidatorId::new("execution.skip"),
            Arc::new(InvalidSkip { missing }),
        ));
        let error = plan
            .validate(
                ReflectedRef::new(&Root { value: "ok".into() }),
                &ValidationOptions::default(),
            )
            .unwrap_err();
        assert_eq!(error.error().kind(), ExecutionErrorKind::AdapterContractViolation);
    }
}

#[test]
fn test_property_reads_and_rule_invocations_each_consume_a_node() {
    GETTER_CALLS.with(|calls| calls.set(0));
    let models = ModelRegistry::try_global().unwrap();
    let graph = StructureResolver::new(ResolveInputs { models, roots: &[] })
        .resolve()
        .unwrap();
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        TypeMetadata::of::<Root>(),
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .unwrap();
    let options = ValidationOptions::builder()
        .max_nodes(NonZeroUsize::new(2).unwrap())
        .build();
    let error = plan
        .validate(ReflectedRef::new(&Root { value: String::new() }), &options)
        .unwrap_err();
    assert_eq!(error.error().kind(), ExecutionErrorKind::TraversalLimit);
    assert!(error.partial_report().violations().is_empty());
    assert_eq!(GETTER_CALLS.with(Cell::get), 1);
}

thread_local! {
    static RULE_CALLS: Cell<usize> = const { Cell::new(0) };
}

#[Model]
struct Fields {
    #[validator(id = "execution.rule")]
    first: String,
    #[validator(id = "execution.rule")]
    second: String,
}
#[ModelImpl]
impl Fields {
    pub fn first(&self) -> &str {
        GETTER_CALLS.with(|calls| calls.set(calls.get() + 1));
        &self.first
    }
    pub fn second(&self) -> &str {
        GETTER_CALLS.with(|calls| calls.set(calls.get() + 1));
        &self.second
    }
}
#[Model]
struct Elements {
    #[element(validator(id = "execution.rule"))]
    values: Vec<String>,
}
#[ModelImpl]
impl Elements {
    pub fn values(&self) -> &[String] {
        GETTER_CALLS.with(|calls| calls.set(calls.get() + 1));
        &self.values
    }
}

#[Model]
struct OwnedField {
    #[validator(id = "execution.rule")]
    value: String,
}

#[ModelImpl]
impl OwnedField {
    pub fn value(&self) -> String {
        self.value.clone()
    }
}

/// Produces the requested public validator outcome while recording actual
/// calls.
struct OutcomeRule;
impl PreparedValidator for OutcomeRule {
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<RuleOutcome, ExecutionError> {
        RULE_CALLS.with(|calls| calls.set(calls.get() + 1));
        let violations = || {
            (0..3)
                .map(|_| Violation::new(ValidatorId::new("execution.rule"), ViolationCode::new("bad")))
                .collect()
        };
        Ok(match value.as_text().unwrap() {
            "invalid" => RuleOutcome::Invalid(violations()),
            "prerequisite" => RuleOutcome::Skipped {
                reason: SkipReason::FailedPrerequisite,
                prerequisites: violations(),
            },
            "missing" => RuleOutcome::Skipped {
                reason: SkipReason::MissingOptional,
                prerequisites: vec![],
            },
            "empty-invalid" => RuleOutcome::Invalid(vec![]),
            "empty-prerequisite" => RuleOutcome::Skipped {
                reason: SkipReason::FailedPrerequisite,
                prerequisites: vec![],
            },
            "bad-missing" => RuleOutcome::Skipped {
                reason: SkipReason::MissingOptional,
                prerequisites: violations(),
            },
            "error" => {
                return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed)
                    .with_source(std::io::Error::other("original validator cause")));
            }
            _ => RuleOutcome::Valid,
        })
    }
}
fn prepare_outcome(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(OutcomeRule))
}
static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare_outcome)];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
static REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("execution.rule"),
    &DESCRIPTOR,
    RegistrationSource::new("execution-tests", "outcomes", file!(), line!()),
);

/// Compiles a fresh explicit root and executes it without sharing counters
/// across threads.
fn run(
    root: &'static TypeMetadata,
    value: ReflectedRef<'_>,
    options: &ValidationOptions,
) -> Result<ValidationReport, ModelValidationError> {
    GETTER_CALLS.with(|calls| calls.set(0));
    RULE_CALLS.with(|calls| calls.set(0));
    let models = ModelRegistry::try_global().unwrap();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs { models, roots: &roots })
        .resolve()
        .unwrap();
    let validators = ValidatorRegistry::from_registrations([REGISTRATION]).unwrap();
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .unwrap();
    assert_eq!(GETTER_CALLS.with(Cell::get), 0);
    plan.validate(value, options)
}

#[test]
fn test_fields_and_selectors_stop_globally_for_every_violation_branch() {
    for outcome in ["invalid", "prerequisite"] {
        let fields = Fields {
            first: outcome.into(),
            second: "invalid".into(),
        };
        let elements = Elements {
            values: vec![outcome.into(), "invalid".into()],
        };
        for (root, value) in [
            (TypeMetadata::of::<Fields>(), ReflectedRef::new(&fields)),
            (TypeMetadata::of::<Elements>(), ReflectedRef::new(&elements)),
        ] {
            for limit in [1, 2] {
                let options = ValidationOptions::builder()
                    .max_violations(NonZeroUsize::new(limit).unwrap())
                    .build();
                let report = run(root, value.clone(), &options).unwrap();
                assert_eq!(report.violations().len(), limit);
                assert!(report.is_truncated());
                assert_eq!(GETTER_CALLS.with(Cell::get), 1);
                assert_eq!(RULE_CALLS.with(Cell::get), 1);
            }
            let report = run(
                root,
                value,
                &ValidationOptions::builder().mode(ValidationMode::FailFast).build(),
            )
            .unwrap();
            assert_eq!(report.violations().len(), 1);
            assert_eq!(RULE_CALLS.with(Cell::get), 1);
            assert_eq!(GETTER_CALLS.with(Cell::get), 1);
        }
    }
}

#[test]
fn test_fields_and_selectors_reject_invalid_outcome_contracts() {
    for outcome in ["empty-invalid", "empty-prerequisite", "bad-missing"] {
        let fields = Fields {
            first: outcome.into(),
            second: "valid".into(),
        };
        let elements = Elements {
            values: vec![outcome.into()],
        };
        for (root, value) in [
            (TypeMetadata::of::<Fields>(), ReflectedRef::new(&fields)),
            (TypeMetadata::of::<Elements>(), ReflectedRef::new(&elements)),
        ] {
            let error = run(root, value, &ValidationOptions::default()).unwrap_err();
            assert_eq!(error.error().kind(), ExecutionErrorKind::AdapterContractViolation);
            assert!(error.partial_report().violations().is_empty());
            assert_eq!(error.root_type_id(), Some(root.type_id()));
            assert_eq!(error.owner_type_id(), Some(root.type_id()));
            assert_eq!(error.field_location().unwrap().index(), 0);
            assert_eq!(error.declared_rule_id(), Some("execution.rule"));
            assert!(error.source().unwrap().downcast_ref::<ExecutionError>().is_some());
        }
    }
}

#[test]
fn test_legal_skip_does_not_trigger_fail_fast_and_occurrences_are_unique() {
    let fields = Fields {
        first: "missing".into(),
        second: "invalid".into(),
    };
    let report = run(
        TypeMetadata::of::<Fields>(),
        ReflectedRef::new(&fields),
        &ValidationOptions::builder().mode(ValidationMode::FailFast).build(),
    )
    .unwrap();
    assert_eq!(RULE_CALLS.with(Cell::get), 2);
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.skipped().len(), 1);
    assert_eq!(report.skipped()[0].occurrence(), 0);
    assert_eq!(report.violations()[0].path().render(), "second");
}

#[test]
fn test_execution_error_retains_partial_report_occurrence_and_original_source() {
    let value = Fields {
        first: "invalid".into(),
        second: "error".into(),
    };
    let error = run(
        TypeMetadata::of::<Fields>(),
        ReflectedRef::new(&value),
        &ValidationOptions::default(),
    )
    .unwrap_err();
    assert_eq!(error.partial_report().violations().len(), 3);
    assert_eq!(error.occurrence(), Some(1));
    assert_eq!(error.field_location().unwrap().owner(), TypeId::of::<Fields>());
    assert_eq!(error.field_location().unwrap().index(), 1);
    assert!(error.declaration().unwrap().line.is_some());
    assert_eq!(error.owner_type_id(), Some(TypeId::of::<Fields>()));
    assert_eq!(error.declared_rule_id(), Some("execution.rule"));
    assert_eq!(error.error().path().render(), "second");
    assert!(!format!("{error}").is_empty());
    assert!(format!("{error:?}").contains("ModelValidationError"));
    assert!(
        error
            .source()
            .unwrap()
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .is_some()
    );
    let (execution_error, partial_report) = error.into_parts();
    assert_eq!(execution_error.path().render(), "second");
    assert_eq!(partial_report.violations().len(), 3);
}

#[test]
fn test_model_validation_error_without_field_context_exposes_empty_accessors() {
    let error = run(
        TypeMetadata::of::<Fields>(),
        ReflectedRef::new(&42_u32),
        &ValidationOptions::default(),
    )
    .unwrap_err();
    assert_eq!(error.root_type_id(), Some(TypeId::of::<Fields>()));
    assert_eq!(error.owner_type_id(), Some(TypeId::of::<Fields>()));
    assert!(error.occurrence().is_none());
    assert!(error.field_location().is_none());
    assert!(error.declaration().is_none());
    assert!(error.declared_rule_id().is_none());
    assert!(error.dependency_object_path().is_none());
    assert!(error.dependency_property_path().is_none());
    let (execution_error, report) = error.into_parts();
    assert_eq!(execution_error.kind(), ExecutionErrorKind::InputTypeMismatch);
    assert!(report.violations().is_empty());
}

#[test]
fn test_owned_getter_value_is_executed_without_borrowing() {
    let value = OwnedField { value: "valid".into() };
    assert!(TypeMetadata::try_of::<OwnedField>().is_ok());
    let report = run(
        TypeMetadata::of::<OwnedField>(),
        ReflectedRef::new(&value),
        &ValidationOptions::default(),
    )
    .expect("owned getter should produce a valid value");
    assert!(report.is_valid());
}

#[test]
fn test_selector_budgets_preserve_partial_report_and_stop_before_next_rule() {
    let value = Elements {
        values: vec!["invalid".into(), "invalid".into()],
    };
    for options in [
        ValidationOptions::builder()
            .max_nodes(NonZeroUsize::new(4).unwrap())
            .build(),
        ValidationOptions::builder()
            .max_comparisons(NonZeroUsize::new(1).unwrap())
            .build(),
    ] {
        let error = run(TypeMetadata::of::<Elements>(), ReflectedRef::new(&value), &options).unwrap_err();
        assert_eq!(error.error().kind(), ExecutionErrorKind::TraversalLimit);
        assert_eq!(error.partial_report().violations().len(), 3);
        assert!(!error.partial_report().is_truncated());
        assert_eq!(error.error().path().render(), "values[1]");
        assert_eq!(RULE_CALLS.with(Cell::get), 1);
    }
    let options = ValidationOptions::builder()
        .max_depth(NonZeroUsize::new(1).unwrap())
        .build();
    let error = run(TypeMetadata::of::<Elements>(), ReflectedRef::new(&value), &options).unwrap_err();
    assert_eq!(error.error().kind(), ExecutionErrorKind::TraversalLimit);
    assert_eq!(RULE_CALLS.with(Cell::get), 0);
    assert!(error.partial_report().violations().is_empty());
}

#[test]
fn test_node_budget_boundary_preserves_completed_field_results() {
    let value = Fields {
        first: "invalid".into(),
        second: "invalid".into(),
    };
    let options = ValidationOptions::builder()
        .max_nodes(NonZeroUsize::new(4).unwrap())
        .build();
    let error = run(TypeMetadata::of::<Fields>(), ReflectedRef::new(&value), &options).unwrap_err();
    assert_eq!(error.partial_report().violations().len(), 3);
    assert_eq!(error.error().kind(), ExecutionErrorKind::TraversalLimit);
    assert_eq!(RULE_CALLS.with(Cell::get), 1);
    assert_eq!(GETTER_CALLS.with(Cell::get), 2);
    let options = ValidationOptions::builder()
        .max_nodes(NonZeroUsize::new(5).unwrap())
        .build();
    assert_eq!(
        run(TypeMetadata::of::<Fields>(), ReflectedRef::new(&value), &options)
            .unwrap()
            .violations()
            .len(),
        6
    );
}

#[Model]
struct OptionalField {
    #[validator(id = "execution.rule")]
    value: Option<String>,
}
#[ModelImpl]
impl OptionalField {
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}
struct Skip;
impl PreparedValidator for Skip {
    fn validate(&self, _: ValidationValue<'_>, _: &BoundValidationContext<'_>) -> Result<RuleOutcome, ExecutionError> {
        Ok(RuleOutcome::Skipped {
            reason: SkipReason::MissingOptional,
            prerequisites: vec![],
        })
    }
}

#[test]
fn test_model_and_field_skip_occurrences_are_unique_without_triggering_fail_fast() {
    let root = TypeMetadata::of::<OptionalField>();
    let models = ModelRegistry::try_global().unwrap();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs { models, roots: &roots })
        .resolve()
        .unwrap();
    let validators = ValidatorRegistry::from_registrations([REGISTRATION]).unwrap();
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .unwrap()
    .with_model_rule(ModelRuleBinding::from_prepared::<OptionalField>(
        ValidatorId::new("execution.skip"),
        Arc::new(Skip),
    ))
    .with_model_rule(ModelRuleBinding::from_prepared::<OptionalField>(
        ValidatorId::new("execution.skip"),
        Arc::new(Skip),
    ));
    let report = plan
        .validate(
            ReflectedRef::new(&OptionalField { value: None }),
            &ValidationOptions::builder().mode(ValidationMode::FailFast).build(),
        )
        .unwrap();
    assert!(report.violations().is_empty());
    assert!(!report.is_truncated());
    assert_eq!(
        report
            .skipped()
            .iter()
            .map(|skip| skip.occurrence())
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
}

thread_local! {
    static PARENT_READS: Cell<usize> = const { Cell::new(0) };
}
#[Model]
struct Parent {
    nickname: String,
}
#[ModelImpl]
impl Parent {
    pub fn nickname(&self) -> &str {
        PARENT_READS.with(|calls| calls.set(calls.get() + 1));
        &self.nickname
    }
}
#[Model]
struct Dependent {
    #[validator(id = "execution.dependent", depends_on(expected(path = "..", property = nickname)))]
    value: String,
}
struct ParentRule;
impl PreparedValidator for ParentRule {
    fn validate(
        &self,
        _: ValidationValue<'_>,
        context: &BoundValidationContext<'_>,
    ) -> Result<RuleOutcome, ExecutionError> {
        assert_eq!(context.value(0)?.as_text(), Some("parent"));
        Ok(RuleOutcome::Valid)
    }
}
fn prepare_parent(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(ParentRule))
}
static PARENT_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    &[DependencySpec::new("expected", InputType::Text, false)],
    prepare_parent,
)];
static PARENT_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(PARENT_SIGNATURES);
static PARENT_REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("execution.dependent"),
    &PARENT_DESCRIPTOR,
    RegistrationSource::new("execution-tests", "parent", file!(), line!()),
);

#[test]
fn test_parent_dependency_reads_share_depth_and_node_budgets_and_keep_navigation_context() {
    let root = TypeMetadata::of::<Dependent>();
    let parent = TypeMetadata::of::<Parent>();
    let models = ModelRegistry::try_global().unwrap();
    let roots = [root, parent];
    let graph = StructureResolver::new(ResolveInputs { models, roots: &roots })
        .resolve()
        .unwrap();
    let validators = ValidatorRegistry::from_registrations([PARENT_REGISTRATION]).unwrap();
    let known = ValidationPlan::build_with_context(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
        &[parent],
    )
    .unwrap();
    let deferred = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .unwrap();
    let value = Dependent { value: "child".into() };
    let containing = Parent {
        nickname: "parent".into(),
    };
    for plan in [&known, &deferred] {
        for options in [
            ValidationOptions::builder()
                .max_depth(NonZeroUsize::new(1).unwrap())
                .build(),
            ValidationOptions::builder()
                .max_nodes(NonZeroUsize::new(2).unwrap())
                .build(),
        ] {
            PARENT_READS.with(|calls| calls.set(0));
            let error = plan
                .validate_with_context(ReflectedRef::new(&value), &[ReflectedRef::new(&containing)], &options)
                .unwrap_err();
            assert_eq!(error.error().kind(), ExecutionErrorKind::TraversalLimit);
            assert_eq!(PARENT_READS.with(Cell::get), 0);
            assert_eq!(error.root_type_id(), Some(TypeId::of::<Dependent>()));
            assert_eq!(error.dependency_object_path().unwrap().to_string(), "..");
            assert_eq!(error.dependency_property_path().unwrap().to_string(), "nickname");
            assert_eq!(error.error().path().render(), "nickname");
        }
        let options = ValidationOptions::builder()
            .max_depth(NonZeroUsize::new(2).unwrap())
            .max_nodes(NonZeroUsize::new(4).unwrap())
            .build();
        assert!(
            plan.validate_with_context(ReflectedRef::new(&value), &[ReflectedRef::new(&containing)], &options)
                .unwrap()
                .is_valid()
        );
        let error = plan
            .validate(ReflectedRef::new(&value), &ValidationOptions::default())
            .unwrap_err();
        assert_eq!(error.error().kind(), ExecutionErrorKind::MissingRequiredDependencyValue);
        assert_eq!(error.dependency_object_path().unwrap().to_string(), "..");
    }
}
