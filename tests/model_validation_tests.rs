// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![cfg(feature = "validation")]

//! Focused model validation execution tests.

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
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_model_metadata::validation::ValidationSelection;
use qubit_reflect::ReflectRegistry;
use qubit_reflect::ReflectedRef;
use qubit_reflect::identity::FragmentIdentity;
use qubit_validator::BindError;
use qubit_validator::BoundValidationContext;
use qubit_validator::ExecutionError;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::RuleOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorSignature;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;

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
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<RuleOutcome, ExecutionError> {
        assert!(value.as_text().is_some());
        Ok(RuleOutcome::Invalid(vec![Violation::new(
            ValidatorId::new("test.reject"),
            ViolationCode::new("value.invalid"),
        )]))
    }
}
fn prepare(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(Reject))
}

struct RejectModel;
impl PreparedValidator for RejectModel {
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<RuleOutcome, ExecutionError> {
        assert!(value.typed::<TestModel>().is_some());
        Ok(RuleOutcome::Invalid(vec![Violation::new(
            ValidatorId::new("test.model.reject"),
            ViolationCode::new("model.invalid"),
        )]))
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
        ModelRuleBinding::from_prepared::<TestModel>(ValidatorId::new("test.model.reject"), Arc::new(RejectModel));
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
    let models = ModelRegistry::from_metadata(&[(metadata, source())]).expect("model registry");
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
