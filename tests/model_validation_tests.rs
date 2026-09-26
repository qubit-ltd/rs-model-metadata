// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![cfg(feature = "validation")]

//! Focused model validation execution tests.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;

use bigdecimal::BigDecimal;
use chrono::DateTime;
use chrono::NaiveDate;
use chrono::NaiveTime;
use chrono::Utc;
use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_model_metadata::metadata::PropertyAccessError;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveErrorKind;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::FieldPath;
use qubit_model_metadata::validation::ModelRuleBinding;
use qubit_model_metadata::validation::ValidationBuildErrorKind;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationMode;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_model_metadata::validation::ValidationSelection;
use qubit_reflect::ReflectRegistry;
use qubit_reflect::ReflectedRef;
use qubit_reflect::identity::FragmentIdentity;
use qubit_validator::BindError;
use qubit_validator::BoundValidationContext;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorSignature;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;
use qubit_validator::ViolationParam;

#[Model(id = "validation.TestModel")]
struct TestModel {
    #[validator(id = "test.reject")]
    name: String,
}

#[Model(id = "validation.SelectorFixture")]
struct SelectorFixture {
    #[sequence(min_items = 0)]
    #[element(validator(id = "test.reject"))]
    values: Vec<String>,
}

#[Model(id = "validation.NestedModel")]
struct NestedModel {
    #[validator(id = "test.reject")]
    name: String,
}

#[Model(id = "validation.NestedRoot")]
struct NestedRoot {
    child: Option<NestedModel>,
}

#[ModelImpl]
impl NestedRoot {
    pub fn child(&self) -> Option<&NestedModel> {
        self.child.as_ref()
    }
}

#[ModelImpl]
impl SelectorFixture {
    pub fn values(&self) -> &[String] {
        &self.values
    }
}

struct Reject;
impl PreparedValidator for Reject {
    fn input_type(&self) -> InputType {
        InputType::Text
    }
    fn dependency_specs(&self) -> &'static [qubit_validator::DependencySpec] {
        &[]
    }
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        assert!(value.as_text().is_some());
        Ok(PreparedOutcome::Invalid(vec![ViolationDraft::new(ViolationCode::new(
            "value.invalid",
        ))]))
    }
}
fn prepare(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(Reject))
}

struct RejectModel;
impl PreparedValidator for RejectModel {
    fn input_type(&self) -> InputType {
        InputType::of::<TestModel>()
    }
    fn dependency_specs(&self) -> &'static [qubit_validator::DependencySpec] {
        &[]
    }
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        assert!(value.typed::<TestModel>().is_some());
        Ok(PreparedOutcome::Invalid(vec![ViolationDraft::new(ViolationCode::new(
            "model.invalid",
        ))]))
    }
}

struct FailModel;
impl PreparedValidator for FailModel {
    fn input_type(&self) -> InputType {
        InputType::of::<TestModel>()
    }
    fn dependency_specs(&self) -> &'static [qubit_validator::DependencySpec] {
        &[]
    }
    fn validate(
        &self,
        _: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed)
            .with_trusted_source(PropertyAccessError::user("sensitive property detail")))
    }
}
static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare)];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
static REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("test.reject"),
    &DESCRIPTOR,
    RegistrationSource::new("model-validation-tests", "test", file!(), line!()),
);

fn source() -> &'static FragmentIdentity {
    Box::leak(Box::new(FragmentIdentity::new(
        "model-validation-tests",
        "test",
        line!(),
        1,
        "test",
        1,
    )))
}

#[test]
fn executes_bound_rule_and_prefixes_field_path() {
    let metadata = TypeMetadata::of::<TestModel>();
    let reflection = ReflectRegistry::initialize().expect("reflection registry");
    let models = ModelRegistry::from_reflect_registry(reflection).expect("model registry");
    assert!(models.by_type_id(TypeMetadata::of::<NestedModel>().type_id()).is_some());
    let graph = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: &models,
    })
    .resolve()
    .expect("structure");
    let validators = ValidatorRegistry::from_registrations([REGISTRATION]).expect("validator registry");
    let plan = ValidationPlan::build(
        metadata,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("binding");
    let report = plan
        .validate(
            ReflectedRef::new(&TestModel { name: "bad".to_owned() }),
            &ValidationOptions::default(),
        )
        .expect("execution");
    assert!(!report.is_valid());
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.violations()[0].path().render(), "name");
}

#[test]
fn executes_typed_model_rule_binding() {
    let metadata = TypeMetadata::of::<TestModel>();
    let reflection = ReflectRegistry::initialize().expect("reflection registry");
    let models = ModelRegistry::from_reflect_registry(reflection).expect("model registry");
    let graph = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: &models,
    })
    .resolve()
    .expect("structure");
    let validators = ValidatorRegistry::from_registrations([REGISTRATION]).expect("validator registry");
    let binding =
        ModelRuleBinding::from_prepared::<TestModel>(ValidatorId::new("test.model.reject"), Arc::new(RejectModel))
            .expect("prepared model shape");
    let diagnostic = format!("{binding:?}");
    assert!(diagnostic.contains("test.model.reject"));
    assert!(
        !diagnostic.contains("RejectModel"),
        "debug output does not inspect the prepared implementation"
    );
    let plan = ValidationPlan::build(
        metadata,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("binding")
    .with_model_rule(binding);
    let report = plan
        .validate(
            ReflectedRef::new(&TestModel { name: "bad".to_owned() }),
            &ValidationOptions::default(),
        )
        .expect("execution");
    assert_eq!(report.violations().len(), 2);
    assert_eq!(report.violations()[0].path().render(), "");
    let empty: [&str; 0] = [];
    for (path, expected) in [(FieldPath::from_segments(empty), ""), (FieldPath::new("name"), "name")] {
        let options = ValidationOptions::builder()
            .selection(ValidationSelection::Fields(vec![path]))
            .build();
        let selected = plan
            .validate(ReflectedRef::new(&TestModel { name: "bad".to_owned() }), &options)
            .expect("selected model or field rule");
        assert_eq!(selected.violations().len(), 1);
        assert_eq!(selected.violations()[0].path().render(), expected);
    }
}

#[test]
fn model_rule_execution_errors_keep_the_bound_rule_id() {
    let metadata = TypeMetadata::of::<TestModel>();
    let reflection = ReflectRegistry::initialize().expect("reflection registry");
    let models = ModelRegistry::from_reflect_registry(reflection).expect("model registry");
    let graph = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: &models,
    })
    .resolve()
    .expect("structure");
    let validators = ValidatorRegistry::from_registrations([REGISTRATION]).expect("validator registry");
    let binding =
        ModelRuleBinding::from_prepared::<TestModel>(ValidatorId::new("test.model.failure"), Arc::new(FailModel))
            .expect("prepared model shape");
    let plan = ValidationPlan::build(
        metadata,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("binding")
    .with_model_rule(binding);

    let error = plan
        .validate(
            ReflectedRef::new(&TestModel {
                name: "value".to_owned(),
            }),
            &ValidationOptions::default(),
        )
        .expect_err("model rule execution fails");
    assert_eq!(error.error().kind(), ExecutionErrorKind::PropertyReadFailed);
    assert_eq!(error.error().rule_id(), Some(ValidatorId::new("test.model.failure")));
    let source = error.error().trusted_source().expect("trusted cause retained");
    assert!(source.is::<PropertyAccessError>());
    assert!(source.to_string().contains("sensitive property detail"));
    assert!(!error.to_string().contains("sensitive property detail"));
    assert!(!format!("{error:?}").contains("sensitive property detail"));
    let standard_source = std::error::Error::source(&error).expect("execution error is standard source");
    assert!(standard_source.is::<ExecutionError>());
    assert!(std::error::Error::source(standard_source).is_none());
}

#[test]
fn executes_element_selector_for_borrowed_slice() {
    let metadata = TypeMetadata::of::<SelectorFixture>();
    let reflection = ReflectRegistry::initialize().expect("reflection registry");
    let models = ModelRegistry::from_reflect_registry(reflection).expect("model registry");
    let graph = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: &models,
    })
    .resolve()
    .expect("structure");
    let validators = ValidatorRegistry::from_registrations([REGISTRATION]).expect("validator registry");
    let plan = ValidationPlan::build(
        metadata,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("selector binding");
    let report = plan
        .validate(
            ReflectedRef::new(&SelectorFixture {
                values: vec!["first".to_owned(), "second".to_owned()],
            }),
            &ValidationOptions::default(),
        )
        .expect("execution");
    assert_eq!(report.violations().len(), 2);
    assert_eq!(report.violations()[1].path().render(), "values[1]");
    let comparison_limited = ValidationOptions::builder()
        .max_comparisons(NonZeroUsize::new(1).expect("non-zero"))
        .build();
    assert!(
        plan.validate(
            ReflectedRef::new(&SelectorFixture {
                values: vec!["first".to_owned(), "second".to_owned()],
            }),
            &comparison_limited,
        )
        .is_err()
    );
}

#[test]
fn executes_validators_declared_by_an_optional_nested_model() {
    let metadata = TypeMetadata::of::<NestedRoot>();
    let reflection = ReflectRegistry::initialize().expect("reflection registry");
    let models = ModelRegistry::from_reflect_registry(reflection).expect("model registry");
    assert!(models.by_type_id(TypeMetadata::of::<NestedModel>().type_id()).is_some());
    let graph = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: &models,
    })
    .resolve()
    .expect("structure");
    let validators = ValidatorRegistry::from_registrations([REGISTRATION]).expect("validator registry");
    let plan = ValidationPlan::build(
        metadata,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("nested binding");
    assert_eq!(plan.binding_count(), 1);

    let report = plan
        .validate(
            ReflectedRef::new(&NestedRoot {
                child: Some(NestedModel { name: "bad".to_owned() }),
            }),
            &ValidationOptions::default(),
        )
        .expect("nested execution");
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.violations()[0].path().render(), "child.name");

    let report = plan
        .validate(
            ReflectedRef::new(&NestedRoot { child: None }),
            &ValidationOptions::default(),
        )
        .expect("missing optional nested value");
    assert!(report.is_valid());
}

#[test]
fn traversal_budgets_are_enforced_before_execution() {
    let metadata = TypeMetadata::of::<TestModel>();
    let models = ModelRegistry::from_static_metadata(&[(metadata, source())]).expect("model registry");
    let graph = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: &models,
    })
    .resolve()
    .expect("structure");
    let validators = ValidatorRegistry::from_registrations([REGISTRATION]).expect("validator registry");
    let plan = ValidationPlan::build(
        metadata,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("binding");
    let model = TestModel { name: "bad".to_owned() };
    let value = ReflectedRef::new(&model);
    let depth = ValidationOptions::builder()
        .max_depth(NonZeroUsize::new(1).expect("non-zero"))
        .build();
    assert!(plan.validate(value.clone(), &depth).is_ok());
    let nodes = ValidationOptions::builder()
        .max_nodes(NonZeroUsize::new(1).expect("non-zero"))
        .build();
    assert!(plan.validate(value, &nodes).is_err());
}
#[Model(no_hash)]
struct MapCountFixture {
    #[map(min_entries = 1, max_entries = 2)]
    hashed: HashMap<String, i32>,
    #[map(min_entries = 1, max_entries = 2)]
    ordered: BTreeMap<String, i32>,
}

#[ModelImpl]
impl MapCountFixture {
    pub fn ordered(&self) -> &BTreeMap<String, i32> {
        &self.ordered
    }
}

#[Model(no_hash)]
struct MapOptionalFixture {
    child: Option<MapCountFixture>,
}

#[ModelImpl]
impl MapOptionalFixture {
    pub fn child(&self) -> Option<&MapCountFixture> {
        self.child.as_ref()
    }
}

#[Model(no_hash)]
struct DirectOptionalMaps {
    #[map(min_entries = 1, max_entries = 2)]
    hashed: Option<HashMap<String, i32>>,
    #[map(min_entries = 1, max_entries = 2)]
    ordered: Option<BTreeMap<String, i32>>,
}

#[Model(no_hash)]
struct OptionalGetterMaps {
    #[map(min_entries = 1, max_entries = 2)]
    hashed: Option<HashMap<String, i32>>,
    #[map(min_entries = 1, max_entries = 2)]
    ordered: Option<BTreeMap<String, i32>>,
}

#[ModelImpl]
impl OptionalGetterMaps {
    pub fn hashed(&self) -> Option<&HashMap<String, i32>> {
        self.hashed.as_ref()
    }
    pub fn ordered(&self) -> Option<&BTreeMap<String, i32>> {
        self.ordered.as_ref()
    }
}

#[Model(no_hash)]
struct MapUnsupportedFixture {
    #[map(min_entries = 1)]
    entries: MapAlias,
}

type MapAlias = HashMap<String, i32>;

fn map_plan(
    root: &'static TypeMetadata,
) -> Result<ValidationPlan<'static>, qubit_model_metadata::validation::ValidationBuildErrors> {
    let roots = Box::leak(Box::new([root]));
    let graph = StructureResolver::new(ResolveInputs {
        roots,
        models: ModelRegistry::global(),
    })
    .resolve()
    .expect("map structure");
    let graph = Box::leak(Box::new(graph));
    let validators = Box::leak(Box::new(ValidatorRegistry::empty()));
    ValidationPlan::build(root, ValidationBuildInputs { graph, validators })
}

#[test]
fn map_count_validates_hash_and_tree_bounds_on_field_paths() {
    let plan = map_plan(TypeMetadata::of::<MapCountFixture>()).expect("map count binding");
    assert_eq!(plan.binding_count(), 2);
    for count in 0..=3 {
        let model = MapCountFixture {
            hashed: (0..count).map(|i| (i.to_string(), i)).collect(),
            ordered: (0..count).map(|i| (i.to_string(), i)).collect(),
        };
        let report = plan
            .validate(ReflectedRef::new(&model), &ValidationOptions::default())
            .expect("map execution");
        assert_eq!(report.is_valid(), (1..=2).contains(&count));
        let paths: Vec<_> = report.violations().iter().map(|v| v.path().render()).collect();
        assert_eq!(
            paths,
            if (1..=2).contains(&count) {
                vec![]
            } else {
                vec!["hashed".to_owned(), "ordered".to_owned()]
            }
        );
        for violation in report.violations() {
            assert_eq!(violation.rule_id().as_str(), "qubit.rules.collection.item_count");
            assert_eq!(
                violation.code().as_str(),
                if count == 0 {
                    "collection.too_small"
                } else {
                    "collection.too_large"
                }
            );
            assert_eq!(
                violation.params().get("bound"),
                Some(&ViolationParam::Unsigned(if count == 0 { 1 } else { 2 }))
            );
        }
    }
}

#[test]
fn map_optional_absence_skips_count_validation() {
    let plan = map_plan(TypeMetadata::of::<MapOptionalFixture>()).expect("optional map binding");
    let model = MapOptionalFixture { child: None };
    let report = plan
        .validate(ReflectedRef::new(&model), &ValidationOptions::default())
        .expect("optional map execution");
    assert!(report.is_valid());
    assert!(report.violations().is_empty());
}

#[test]
fn map_direct_optional_some_and_none_follow_count_bounds() {
    let plan = map_plan(TypeMetadata::of::<DirectOptionalMaps>()).expect("direct optional map binding");
    for count in [None, Some(0), Some(1), Some(2), Some(3)] {
        let model = DirectOptionalMaps {
            hashed: count.map(|n| (0..n).map(|i| (i.to_string(), i)).collect()),
            ordered: count.map(|n| (0..n).map(|i| (i.to_string(), i)).collect()),
        };
        let report = plan
            .validate(ReflectedRef::new(&model), &ValidationOptions::default())
            .expect("direct optional execution");
        let expected_paths = if matches!(count, Some(0 | 3)) {
            vec!["hashed".to_owned(), "ordered".to_owned()]
        } else {
            vec![]
        };
        assert_eq!(
            report
                .violations()
                .iter()
                .map(|v| v.path().render())
                .collect::<Vec<_>>(),
            expected_paths
        );
    }
}

#[test]
fn map_optional_borrowed_getters_validate_some_and_skip_none() {
    let plan = map_plan(TypeMetadata::of::<OptionalGetterMaps>()).expect("optional getter map binding");
    for count in [None, Some(0), Some(1), Some(2), Some(3)] {
        let model = OptionalGetterMaps {
            hashed: count.map(|n| (0..n).map(|i| (i.to_string(), i)).collect()),
            ordered: count.map(|n| (0..n).map(|i| (i.to_string(), i)).collect()),
        };
        let report = plan
            .validate(ReflectedRef::new(&model), &ValidationOptions::default())
            .expect("optional getter execution");
        let expected = if matches!(count, Some(0 | 3)) { 2 } else { 0 };
        assert_eq!(report.violations().len(), expected);
        for violation in report.violations() {
            assert!(matches!(violation.path().render().as_str(), "hashed" | "ordered"));
            assert_eq!(violation.rule_id().as_str(), "qubit.rules.collection.item_count");
        }
    }
}

#[test]
fn map_missing_count_adapter_is_rejected_at_build() {
    let errors = match map_plan(TypeMetadata::of::<MapUnsupportedFixture>()) {
        Ok(_) => panic!("unsupported map shape must fail"),
        Err(errors) => errors,
    };
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
    assert_eq!(errors[0].path(), Some("entries"));
}
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

#[Model]
struct ConstrainedChild {
    #[text(min_chars = 3)]
    name: String,
}

#[Model]
struct ConstrainedRoot {
    child: Option<ConstrainedChild>,
}

#[ModelImpl]
impl ConstrainedRoot {
    /// Exposes the optional nested borrow to the erased execution adapter.
    pub fn child(&self) -> Option<&ConstrainedChild> {
        self.child.as_ref()
    }
}

/// Automatic nested traversal retains standard constraints and optional
/// absence.
#[test]
fn nested_standard_constraints_are_executed() {
    let metadata = TypeMetadata::of::<ConstrainedRoot>();
    let roots = [metadata];
    let graph = StructureResolver::new(ResolveInputs {
        models: ModelRegistry::global(),
        roots: &roots,
    })
    .resolve()
    .expect("structure");
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        metadata,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("nested binding");
    let invalid = ConstrainedRoot {
        child: Some(ConstrainedChild { name: "x".to_owned() }),
    };
    let report = plan
        .validate(ReflectedRef::new(&invalid), &ValidationOptions::default())
        .expect("execution");
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.violations()[0].path().render(), "child.name");
    assert!(
        plan.validate(
            ReflectedRef::new(&ConstrainedRoot { child: None }),
            &ValidationOptions::default()
        )
        .expect("absent child")
        .is_valid()
    );
}

#[Model]
struct UniqueFixture {
    #[sequence(min_items = 4, unique_items)]
    values: Vec<i32>,
    #[text(non_blank)]
    later: String,
}

#[ModelImpl]
impl UniqueFixture {
    pub fn values(&self) -> &[i32] {
        &self.values
    }
}

#[Model]
struct UniqueOnlyFixture {
    #[sequence(unique_items)]
    values: Vec<i32>,
}

#[Model]
struct UniqueTextFixture {
    #[sequence(unique_items)]
    values: Vec<String>,
}

#[ModelImpl]
impl UniqueTextFixture {
    pub fn values(&self) -> &[String] {
        &self.values
    }
}

#[ModelImpl]
impl UniqueOnlyFixture {
    pub fn values(&self) -> &[i32] {
        &self.values
    }
}

#[Model]
struct UniqueArrayFixture {
    #[sequence(unique_items)]
    values: [i32; 3],
}

#[ModelImpl]
impl UniqueArrayFixture {
    pub fn values(&self) -> &[i32] {
        &self.values
    }
}

#[Model]
struct UniqueUnreadableArray {
    #[sequence(unique_items)]
    values: [i32; 3],
}

#[Model]
struct UniqueWrongGetterShape {
    #[sequence(unique_items)]
    values: Vec<i32>,
}

#[Model]
struct UniqueWrongElementType {
    #[sequence(unique_items)]
    values: Vec<i32>,
    bytes: Vec<u8>,
}

#[ModelImpl]
impl UniqueWrongElementType {
    pub fn values(&self) -> &[u8] {
        &self.bytes
    }
}

#[ModelImpl]
impl UniqueWrongGetterShape {
    pub fn values(&self) -> &Vec<i32> {
        &self.values
    }
}

#[test]
fn unique_first_duplicate_has_indexed_path_and_safe_parameter() {
    let plan = map_plan(TypeMetadata::of::<UniqueOnlyFixture>()).expect("unique binding");
    let model = UniqueOnlyFixture {
        values: vec![2, 3, 3, 2],
    };
    let report = plan
        .validate(ReflectedRef::new(&model), &ValidationOptions::default())
        .expect("unique execution");
    assert_eq!(report.violations().len(), 1);
    let violation = &report.violations()[0];
    assert_eq!(violation.rule_id().as_str(), "qubit.rules.collection.unique");
    assert_eq!(violation.code().as_str(), "collection.duplicate_item");
    assert_eq!(violation.path().render(), "values[2]");
    assert_eq!(
        violation.params().get("first_index"),
        Some(&ViolationParam::Unsigned(1))
    );
    for values in [vec![], vec![2], vec![2, 3]] {
        let model = UniqueOnlyFixture { values };
        assert!(
            plan.validate(ReflectedRef::new(&model), &ValidationOptions::default())
                .expect("distinct values")
                .is_valid()
        );
    }
}

#[test]
fn unique_report_does_not_expose_repeated_element() {
    let plan = map_plan(TypeMetadata::of::<UniqueTextFixture>()).expect("text equality binding");
    let model = UniqueTextFixture {
        values: vec!["sensitive-duplicate".to_owned(), "sensitive-duplicate".to_owned()],
    };
    let report = plan
        .validate(ReflectedRef::new(&model), &ValidationOptions::default())
        .expect("duplicate report");
    assert_eq!(report.violations()[0].path().render(), "values[1]");
    assert!(!format!("{:?}", report.violations()[0]).contains("sensitive-duplicate"));
}

#[test]
fn unique_count_and_unique_have_separate_occurrences_and_partial_report() {
    let plan = map_plan(TypeMetadata::of::<UniqueFixture>()).expect("count and unique binding");
    assert_eq!(plan.binding_count(), 3);
    let model = UniqueFixture {
        values: vec![2, 3, 2],
        later: "ok".to_owned(),
    };
    let report = plan
        .validate(ReflectedRef::new(&model), &ValidationOptions::default())
        .expect("all violations");
    assert_eq!(report.violations().len(), 2);
    assert_eq!(report.violations()[0].code().as_str(), "collection.too_small");
    assert_eq!(report.violations()[0].path().render(), "values");
    assert_eq!(report.violations()[1].path().render(), "values[2]");

    let options = ValidationOptions::builder()
        .max_comparisons(NonZeroUsize::new(1).expect("positive"))
        .build();
    let error = plan
        .validate(ReflectedRef::new(&model), &options)
        .expect_err("second comparison exceeds budget");
    assert_eq!(error.error().kind(), ExecutionErrorKind::TraversalLimit);
    assert_eq!(error.occurrence(), Some(1));
    assert_eq!(error.error().path().render(), "values[2]");
    assert_eq!(error.partial_report().violations().len(), 1);
    assert_eq!(
        error.partial_report().violations()[0].code().as_str(),
        "collection.too_small"
    );
}

#[test]
fn unique_fail_fast_stops_before_later_field() {
    let plan = map_plan(TypeMetadata::of::<UniqueFixture>()).expect("unique binding");
    let model = UniqueFixture {
        values: vec![1, 2, 1, 3],
        later: String::new(),
    };
    let options = ValidationOptions::builder().mode(ValidationMode::FailFast).build();
    let report = plan.validate(ReflectedRef::new(&model), &options).expect("fail fast");
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.violations()[0].code().as_str(), "collection.duplicate_item");
    assert_eq!(report.violations()[0].path().render(), "values[2]");
}

#[test]
fn unique_node_budget_stops_before_unfunded_element_read() {
    let plan = map_plan(TypeMetadata::of::<UniqueOnlyFixture>()).expect("unique binding");
    let model = UniqueOnlyFixture { values: vec![1, 2] };
    let options = ValidationOptions::builder()
        .max_nodes(NonZeroUsize::new(4).expect("positive"))
        .build();
    let error = plan
        .validate(ReflectedRef::new(&model), &options)
        .expect_err("second element read exceeds budget");
    assert_eq!(error.error().kind(), ExecutionErrorKind::TraversalLimit);
    assert_eq!(error.error().path().render(), "values[1]");
}

#[test]
fn unique_array_slice_getter_executes_and_unreadable_array_fails_build() {
    let plan = map_plan(TypeMetadata::of::<UniqueArrayFixture>()).expect("array slice getter");
    let model = UniqueArrayFixture { values: [1, 2, 1] };
    let report = plan
        .validate(ReflectedRef::new(&model), &ValidationOptions::default())
        .expect("array execution");
    assert_eq!(report.violations()[0].path().render(), "values[2]");
    let errors = match map_plan(TypeMetadata::of::<UniqueUnreadableArray>()) {
        Ok(_) => panic!("unreadable array must fail at build"),
        Err(errors) => errors,
    };
    assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
    assert_eq!(errors[0].path(), Some("values"));
    let errors = match map_plan(TypeMetadata::of::<UniqueWrongGetterShape>()) {
        Ok(_) => panic!("non-slice getter must fail at build"),
        Err(errors) => errors,
    };
    assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
    assert_eq!(errors[0].path(), Some("values"));
}

#[Model(no_redact, no_display, no_debug, no_serialize, no_deserialize)]
struct ScalarDecimalFixture {
    #[decimal(precision = 4, scale = 2, min = "1.25", max = "9.75", min_inclusive = false)]
    amount: BigDecimal,
}

#[Model(no_redact, no_display, no_debug, no_serialize, no_deserialize)]
struct DecimalCapacityFixture {
    #[decimal(precision = 3, scale = 2)]
    amount: BigDecimal,
}

#[Model(no_redact, no_display, no_debug, no_serialize, no_deserialize)]
struct ScalarTimeFixture {
    #[time(precision = second)]
    instant: DateTime<Utc>,
    #[time(precision = millisecond)]
    local: NaiveTime,
}

#[ModelImpl]
impl ScalarTimeFixture {
    pub fn instant(&self) -> &DateTime<Utc> {
        &self.instant
    }
}

#[Model(no_redact, no_display, no_debug, no_serialize, no_deserialize)]
struct ScalarOptionalFixture {
    #[decimal(scale = 2)]
    amount: Option<BigDecimal>,
    #[time(precision = second)]
    instant: Option<DateTime<Utc>>,
}

#[Model(no_redact, no_display, no_debug, no_serialize, no_deserialize)]
struct ScalarOrderFixture {
    #[decimal(scale = 2)]
    amount: BigDecimal,
    #[text(non_blank)]
    label: String,
}

#[Model(no_redact, no_display, no_debug, no_serialize, no_deserialize)]
struct ScalarUnsupportedTimeFixture {
    #[time(precision = second)]
    date: NaiveDate,
}

#[test]
fn scalar_decimal_normalizes_scale_and_checks_precision_and_exact_range() {
    let plan = map_plan(TypeMetadata::of::<ScalarDecimalFixture>()).expect("decimal binding");
    for (literal, expected) in [
        ("1.2500", Some("decimal.range")),
        ("1.23", Some("decimal.range")),
        ("9.75", None),
        ("9.76", Some("decimal.range")),
        ("1.234", Some("decimal.scale")),
        ("123.45", Some("decimal.precision")),
    ] {
        let model = ScalarDecimalFixture {
            amount: literal.parse().expect("decimal literal"),
        };
        let report = plan
            .validate(ReflectedRef::new(&model), &ValidationOptions::default())
            .expect("decimal execution");
        assert_eq!(
            report.violations().first().map(|v| v.code().as_str()),
            expected,
            "{literal}"
        );
        assert!(report.violations().iter().all(|v| v.path().render() == "amount"));
        assert!(!format!("{report:?}").contains(literal));
    }
}

#[test]
fn scalar_decimal_precision_is_total_capacity_at_declared_scale() {
    let plan = map_plan(TypeMetadata::of::<DecimalCapacityFixture>()).expect("decimal capacity binding");
    for (literal, expected) in [
        ("1.2300", None),
        ("12", Some("decimal.precision")),
        ("1.234", Some("decimal.scale")),
    ] {
        let model = DecimalCapacityFixture {
            amount: literal.parse().expect("decimal literal"),
        };
        let report = plan
            .validate(ReflectedRef::new(&model), &ValidationOptions::default())
            .expect("decimal execution");
        assert_eq!(
            report.violations().first().map(|v| v.code().as_str()),
            expected,
            "{literal}"
        );
        if let Some(violation) = report.violations().first() {
            assert_eq!(violation.rule_id().as_str(), "qubit.rules.decimal.value");
            assert_eq!(violation.path().render(), "amount");
        }
    }
}

#[test]
fn scalar_time_resolutions_apply_to_utc_and_naive_getters() {
    let plan = map_plan(TypeMetadata::of::<ScalarTimeFixture>()).expect("time binding");
    let model = ScalarTimeFixture {
        instant: DateTime::parse_from_rfc3339("2026-09-26T08:00:00.001Z")
            .expect("instant")
            .with_timezone(&Utc),
        local: "08:00:00.001001".parse().expect("naive time"),
    };
    let report = plan
        .validate(ReflectedRef::new(&model), &ValidationOptions::default())
        .expect("time execution");
    let paths: Vec<_> = report.violations().iter().map(|v| v.path().render()).collect();
    assert_eq!(paths, ["instant", "local"]);
    assert!(
        report
            .violations()
            .iter()
            .all(|v| v.code().as_str() == "time.precision")
    );
    assert!(
        report
            .violations()
            .iter()
            .all(|v| v.rule_id().as_str() == "qubit.rules.time.precision")
    );
}

#[test]
fn scalar_optional_none_skips_both_rules() {
    let plan = map_plan(TypeMetadata::of::<ScalarOptionalFixture>()).expect("optional scalar binding");
    let model = ScalarOptionalFixture {
        amount: None,
        instant: None,
    };
    let report = plan
        .validate(ReflectedRef::new(&model), &ValidationOptions::default())
        .expect("optional scalar execution");
    assert!(report.is_valid());
    assert_eq!(report.violations().len(), 0);

    let present = ScalarOptionalFixture {
        amount: Some("1.234".parse().expect("decimal literal")),
        instant: Some(
            DateTime::parse_from_rfc3339("2026-09-26T08:00:00.001Z")
                .expect("instant")
                .with_timezone(&Utc),
        ),
    };
    let report = plan
        .validate(ReflectedRef::new(&present), &ValidationOptions::default())
        .expect("present scalar execution");
    let codes: Vec<_> = report.violations().iter().map(|v| v.code().as_str()).collect();
    assert_eq!(codes, ["decimal.scale", "time.precision"]);
}

#[test]
fn scalar_declaration_reports_before_unrelated_field_failure() {
    let plan = map_plan(TypeMetadata::of::<ScalarOrderFixture>()).expect("ordered scalar binding");
    let model = ScalarOrderFixture {
        amount: "1.234".parse().expect("decimal literal"),
        label: String::new(),
    };
    let report = plan
        .validate(ReflectedRef::new(&model), &ValidationOptions::default())
        .expect("ordered execution");
    let codes: Vec<_> = report.violations().iter().map(|v| v.code().as_str()).collect();
    let paths: Vec<_> = report.violations().iter().map(|v| v.path().render()).collect();
    assert_eq!(codes, ["decimal.scale", "text.blank"]);
    assert_eq!(paths, ["amount", "label"]);
}

#[test]
fn scalar_unsupported_temporal_type_is_rejected_at_build() {
    let errors = match map_plan(TypeMetadata::of::<ScalarUnsupportedTimeFixture>()) {
        Ok(_) => panic!("date-only time rule must fail at build"),
        Err(errors) => errors,
    };
    assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
    assert_eq!(errors[0].path(), Some("date"));
}

#[test]
fn unique_mismatched_getter_element_type_fails_at_build() {
    let root = TypeMetadata::of::<UniqueWrongElementType>();
    let roots = [root];
    let errors = StructureResolver::new(ResolveInputs {
        roots: &roots,
        models: ModelRegistry::global(),
    })
    .resolve()
    .expect_err("getter element type must match the declared sequence element");
    assert_eq!(errors.errors().len(), 1);
    assert_eq!(errors.errors()[0].kind(), ResolveErrorKind::InvalidProperties);
    assert_eq!(errors.errors()[0].path().expect("getter field").to_string(), "values");
}
