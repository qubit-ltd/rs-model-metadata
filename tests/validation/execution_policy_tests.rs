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
use qubit_model_metadata::validation::FieldPath;
use qubit_model_metadata::validation::ModelRuleBinding;
use qubit_model_metadata::validation::ModelValidationError;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationMode;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_model_metadata::validation::ValidationSelection;
use qubit_reflect::ReflectedRef;
use qubit_validator::BindError;
use qubit_validator::BoundValidationContext;
use qubit_validator::DependencySpec;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::SkipReason;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorSignature;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;

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

struct Many;
impl PreparedValidator for Many {
    fn validate(
        &self,
        _: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        let nested_path = ValidationPath::root().with_field("nested");
        let drafts = (0..3)
            .map(|_| ViolationDraft::new(ViolationCode::new("bad")).with_path(nested_path.clone()))
            .collect();
        Ok(PreparedOutcome::Invalid(drafts))
    }
}

/// Runs a model rule before a field whose getter records whether it was read.
fn assert_stopped(options: ValidationOptions, expected: usize) {
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
        Arc::new(Many),
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
    assert_stopped(ValidationOptions::builder().mode(ValidationMode::FailFast).build(), 1);
}

#[test]
fn test_violation_limit_stops_before_fields() {
    assert_stopped(
        ValidationOptions::builder()
            .max_violations(NonZeroUsize::new(1).unwrap())
            .build(),
        1,
    );
}

#[test]
fn test_exact_limit_stops_before_later_selected_work() {
    assert_stopped(
        ValidationOptions::builder()
            .max_violations(NonZeroUsize::new(3).unwrap())
            .build(),
        3,
    );
}

#[test]
fn test_exact_limit_on_last_selected_occurrence_is_not_truncated() {
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
        Arc::new(Many),
    ));
    let options = ValidationOptions::builder()
        .selection(ValidationSelection::Fields(vec![FieldPath::from_segments(
            std::iter::empty::<String>(),
        )]))
        .max_violations(NonZeroUsize::new(3).unwrap())
        .build();
    let report = plan
        .validate(ReflectedRef::new(&Root { value: "valid".into() }), &options)
        .unwrap();

    assert_eq!(report.failure_count(), 3);
    assert!(!report.is_truncated());
    assert_eq!(GETTER_CALLS.with(Cell::get), 0);
}

struct EmptyInvalid;
impl PreparedValidator for EmptyInvalid {
    fn validate(
        &self,
        _: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        Ok(PreparedOutcome::Invalid(vec![]))
    }
}

#[test]
fn test_model_empty_invalid_outcome_is_an_execution_error() {
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
        ValidatorId::new("execution.empty"),
        Arc::new(EmptyInvalid),
    ));
    let error = plan
        .validate(
            ReflectedRef::new(&Root { value: "ok".into() }),
            &ValidationOptions::default(),
        )
        .unwrap_err();
    assert_eq!(error.error().kind(), ExecutionErrorKind::AdapterContractViolation);
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
    ) -> Result<PreparedOutcome, ExecutionError> {
        RULE_CALLS.with(|calls| calls.set(calls.get() + 1));
        let drafts = || {
            (0..3)
                .map(|_| {
                    ViolationDraft::new(ViolationCode::new("bad"))
                        .with_path(ValidationPath::root().with_field("nested"))
                })
                .collect()
        };
        let prerequisites = || {
            (0..3)
                .map(|_| {
                    Violation::new(ValidatorId::new("execution.rule"), ViolationCode::new("prerequisite"))
                        .with_path(ValidationPath::root().with_field("source").with_field("nested"))
                })
                .collect()
        };
        Ok(match value.as_text().unwrap() {
            "invalid" => PreparedOutcome::Invalid(drafts()),
            "empty-invalid" => PreparedOutcome::Invalid(vec![]),
            "prerequisite" => PreparedOutcome::Skipped {
                reason: SkipReason::FailedPrerequisite,
                prerequisites: prerequisites(),
            },
            "error" => {
                return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed));
            }
            _ => PreparedOutcome::Valid,
        })
    }
}

#[test]
fn test_fields_and_selectors_preserve_prerequisite_paths() {
    let fields = Fields {
        first: "prerequisite".into(),
        second: "valid".into(),
    };
    let elements = Elements {
        values: vec!["prerequisite".into()],
    };
    for (root, value, occurrence_path) in [
        (TypeMetadata::of::<Fields>(), ReflectedRef::new(&fields), "first"),
        (
            TypeMetadata::of::<Elements>(),
            ReflectedRef::new(&elements),
            "values[0]",
        ),
    ] {
        let report = run(root, value, &ValidationOptions::default()).unwrap();
        assert!(report.violations().is_empty());
        assert_eq!(report.skipped().len(), 1);
        let skipped = &report.skipped()[0];
        assert_eq!(skipped.path().render(), occurrence_path);
        assert_eq!(skipped.prerequisites().len(), 3);
        assert!(
            skipped
                .prerequisites()
                .iter()
                .all(|violation| violation.path().render() == "source.nested")
        );
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
fn test_fields_and_selectors_stop_globally_after_invalid_outcomes() {
    let fields = Fields {
        first: "invalid".into(),
        second: "invalid".into(),
    };
    let elements = Elements {
        values: vec!["invalid".into(), "invalid".into()],
    };
    for (root, value) in [
        (TypeMetadata::of::<Fields>(), ReflectedRef::new(&fields)),
        (TypeMetadata::of::<Elements>(), ReflectedRef::new(&elements)),
    ] {
        let expected_path = if root.type_id() == TypeId::of::<Fields>() {
            "first.nested"
        } else {
            "values[0].nested"
        };
        for limit in [1, 2] {
            let options = ValidationOptions::builder()
                .max_violations(NonZeroUsize::new(limit).unwrap())
                .build();
            let report = run(root, value.clone(), &options).unwrap();
            assert_eq!(report.violations().len(), limit);
            assert_eq!(report.violations()[0].path().render(), expected_path);
            assert_eq!(report.failure_count(), limit);
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
        assert_eq!(report.violations()[0].path().render(), expected_path);
        assert_eq!(report.failure_count(), 1);
        assert_eq!(RULE_CALLS.with(Cell::get), 1);
        assert_eq!(GETTER_CALLS.with(Cell::get), 1);
    }
}

#[test]
fn test_fields_and_selectors_reject_empty_invalid_outcomes() {
    let fields = Fields {
        first: "empty-invalid".into(),
        second: "valid".into(),
    };
    let elements = Elements {
        values: vec!["empty-invalid".into()],
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

#[test]
fn test_missing_optional_does_not_trigger_fail_fast_before_invalid_field() {
    let fields = OptionalFields {
        first: None,
        second: None,
        third: Some("invalid".into()),
    };
    let report = run(
        TypeMetadata::of::<OptionalFields>(),
        ReflectedRef::new(&fields),
        &ValidationOptions::builder().mode(ValidationMode::FailFast).build(),
    )
    .unwrap();
    assert_eq!(RULE_CALLS.with(Cell::get), 1);
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.skipped().len(), 2);
    assert_eq!(report.skipped()[0].occurrence(), 0);
    assert_eq!(report.skipped()[1].occurrence(), 1);
    assert_eq!(report.skipped()[0].reason(), SkipReason::MissingOptional);
    assert_eq!(report.skipped()[1].reason(), SkipReason::MissingOptional);
    assert_eq!(report.skipped()[0].path().render(), "first");
    assert_eq!(report.skipped()[1].path().render(), "second");
    assert_eq!(report.violations()[0].path().render(), "third.nested");
}

#[test]
fn test_execution_error_retains_partial_report_occurrence_without_source() {
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
    assert!(error.source().unwrap().source().is_none());
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
struct OptionalFields {
    #[validator(id = "execution.rule")]
    first: Option<String>,
    #[validator(id = "execution.rule")]
    second: Option<String>,
    #[validator(id = "execution.rule")]
    third: Option<String>,
}
#[ModelImpl]
impl OptionalFields {
    pub fn first(&self) -> Option<&str> {
        self.first.as_deref()
    }
    pub fn second(&self) -> Option<&str> {
        self.second.as_deref()
    }
    pub fn third(&self) -> Option<&str> {
        self.third.as_deref()
    }
}

#[test]
fn test_missing_optional_occurrences_are_unique_without_triggering_fail_fast() {
    let report = run(
        TypeMetadata::of::<OptionalFields>(),
        ReflectedRef::new(&OptionalFields {
            first: None,
            second: None,
            third: None,
        }),
        &ValidationOptions::builder().mode(ValidationMode::FailFast).build(),
    )
    .expect("absent optional fields should be skipped");
    assert!(report.is_valid());
    assert!(!report.is_truncated());
    assert_eq!(RULE_CALLS.with(Cell::get), 0);
    assert_eq!(
        report
            .skipped()
            .iter()
            .map(|skip| skip.occurrence())
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    for skipped in report.skipped() {
        assert_eq!(skipped.reason(), SkipReason::MissingOptional);
        assert!(skipped.prerequisites().is_empty());
    }
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
    ) -> Result<PreparedOutcome, ExecutionError> {
        assert_eq!(context.value(0)?.as_text(), Some("parent"));
        Ok(PreparedOutcome::Valid)
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
