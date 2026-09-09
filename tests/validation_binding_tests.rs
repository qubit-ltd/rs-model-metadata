// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![cfg(feature = "validation")]

//! Binding-only validation plan tests.

use std::sync::Arc;

use qubit_model_derive::Model;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ContextRequirement;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::ReflectedRef;
use qubit_reflect::identity::FragmentIdentity;
use qubit_validator::ArgumentReader;
use qubit_validator::BindError;
use qubit_validator::BoundValidationContext;
use qubit_validator::DependencySpec;
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

#[Model(id = "test.Owner")]
struct Owner {
    kind: u8,
}

#[Model(id = "test.BindingFixture")]
struct BindingFixture {
    owner: Owner,
    #[validator(id = "test.text")]
    value: String,
}

struct AcceptText;

impl PreparedValidator for AcceptText {
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _context: &BoundValidationContext<'_>,
    ) -> Result<RuleOutcome, ExecutionError> {
        assert!(value.as_text().is_some());
        Ok(RuleOutcome::Valid)
    }
}

fn prepare_text(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(AcceptText))
}

static TEXT_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare_text)];
static TEXT_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(TEXT_SIGNATURES);
static TEXT_REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("test.text"),
    &TEXT_DESCRIPTOR,
    RegistrationSource::new("validation-binding-tests", "fixture", file!(), line!()),
);

fn source() -> &'static FragmentIdentity {
    Box::leak(Box::new(FragmentIdentity::new(
        "validation-binding-tests",
        "fixture",
        line!(),
        1,
        "fixture",
        1,
    )))
}

fn inputs<'a>(models: &'a ModelRegistry<'a>) -> ResolveInputs<'a> {
    ResolveInputs { roots: &[], models }
}

#[test]
fn structure_resolution_and_binding_are_separate() {
    let owner = TypeMetadata::of::<Owner>();
    let fixture = TypeMetadata::of::<BindingFixture>();
    let models =
        ModelRegistry::from_metadata(&[(owner, source()), (fixture, source())]).expect("isolated model registry");
    let validators = ValidatorRegistry::from_registrations([TEXT_REGISTRATION]).expect("isolated validator registry");
    let graph = StructureResolver::new(inputs(&models))
        .resolve()
        .expect("structure does not require validator lookup");

    let plan = ValidationPlan::build(
        fixture,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("text declaration binds against the local registry");
    assert_eq!(plan.binding_count(), 1);
}

#[Model]
struct Parent {
    nickname: String,
}

#[Model]
struct Child {
    nickname: String,
    #[validator(id = "test.parent", depends_on(expected(path = "..", property = nickname)))]
    value: String,
}

struct CheckParent;

impl PreparedValidator for CheckParent {
    /// Verifies that parent navigation did not silently select the child field.
    fn validate(
        &self,
        _: ValidationValue<'_>,
        context: &BoundValidationContext<'_>,
    ) -> Result<RuleOutcome, ExecutionError> {
        assert_eq!(context.value(0)?.as_text(), Some("parent"));
        Ok(RuleOutcome::Valid)
    }
}

/// Prepares an isolated validator consuming one text dependency.
fn prepare_parent(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(CheckParent))
}

static PARENT_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    &[DependencySpec::new("expected", InputType::Text, false)],
    prepare_parent,
)];
static PARENT_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(PARENT_SIGNATURES);
static PARENT_REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("test.parent"),
    &PARENT_DESCRIPTOR,
    RegistrationSource::new("validation-binding-tests", "parent", file!(), line!()),
);

/// Parent dependencies require explicit type and instance contexts.
#[test]
fn parent_dependency_uses_explicit_context() {
    let models = ModelRegistry::from_metadata(&[]).expect("empty registry");
    let child = TypeMetadata::of::<Child>();
    let parent = TypeMetadata::of::<Parent>();
    let roots = [child, parent];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("graph");
    assert_eq!(
        graph.dependencies()[0].context_requirement(),
        ContextRequirement::ParentObject
    );
    let validators = ValidatorRegistry::from_registrations([PARENT_REGISTRATION]).expect("registry");
    let plan = ValidationPlan::build_with_context(
        child,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
        &[parent],
    )
    .expect("parent binding");
    let deferred = ValidationPlan::build(
        child,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("parent type may be supplied at execution");
    let value = Child {
        nickname: "child".into(),
        value: "value".into(),
    };
    let parent_value = Parent {
        nickname: "parent".into(),
    };
    let options = ValidationOptions::default();
    assert!(plan.validate(ReflectedRef::new(&value), &options).is_err());
    assert!(deferred.validate(ReflectedRef::new(&value), &options).is_err());
    deferred
        .validate_with_context(ReflectedRef::new(&value), &[ReflectedRef::new(&parent_value)], &options)
        .expect("deferred parent execution");
    plan.validate_with_context(ReflectedRef::new(&value), &[ReflectedRef::new(&parent_value)], &options)
        .expect("parent execution");
}
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

#[Model]
struct RepeatedRules {
    #[validator(id = "test.repeat", params(order = 1))]
    #[validator(id = "test.repeat", params(order = 2))]
    value: String,
}

static EXECUTION_ORDER: std::sync::Mutex<Vec<u32>> = std::sync::Mutex::new(Vec::new());

struct RecordOrder(u32);

impl PreparedValidator for RecordOrder {
    fn validate(&self, _: ValidationValue<'_>, _: &BoundValidationContext<'_>) -> Result<RuleOutcome, ExecutionError> {
        EXECUTION_ORDER.lock().expect("execution log").push(self.0);
        Ok(RuleOutcome::Valid)
    }
}

fn prepare_order(arguments: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(arguments)?;
    let order = reader.required_u32("order")?;
    reader.finish()?;
    Ok(Arc::new(RecordOrder(order)))
}

/// Equal strategy IDs retain distinct parameters and declaration-order
/// execution.
#[test]
fn repeated_validator_ids_execute_every_occurrence() {
    static DESCRIPTOR: ValidatorDescriptor =
        ValidatorDescriptor::new(&[ValidatorSignature::new(InputType::Text, &[], prepare_order)]);
    let registration = ValidatorRegistration::new(
        ValidatorId::new("test.repeat"),
        &DESCRIPTOR,
        RegistrationSource::new("test", "repeat", file!(), line!()),
    );
    let validators = ValidatorRegistry::from_registrations([registration]).expect("validator registry");
    let root = TypeMetadata::of::<RepeatedRules>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs {
        models: ModelRegistry::global(),
        roots: &roots,
    })
    .resolve()
    .expect("structure");
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("binding");
    assert_eq!(plan.binding_count(), 2);
    EXECUTION_ORDER.lock().expect("execution log").clear();
    plan.validate(
        ReflectedRef::new(&RepeatedRules {
            value: "text".to_owned(),
        }),
        &ValidationOptions::default(),
    )
    .expect("execution");
    assert_eq!(*EXECUTION_ORDER.lock().expect("execution log"), [1, 2]);
}
