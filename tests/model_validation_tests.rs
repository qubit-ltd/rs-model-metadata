#![cfg(feature = "validation")]

//! Focused model validation execution tests.

use std::num::NonZeroUsize;
use std::sync::Arc;

use qubit_codec::ValueCodecRegistry;
use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_model_metadata::FragmentIdentity;
use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::ModelResolver;
use qubit_model_metadata::ModelRuleBinding;
use qubit_model_metadata::ReflectRegistry;
use qubit_model_metadata::ReflectedRef;
use qubit_model_metadata::ResolveInputs;
use qubit_model_metadata::TypeMetadata;
use qubit_model_metadata::ValidationBuildInputs;
use qubit_model_metadata::ValidationOptions;
use qubit_model_metadata::ValidationPlan;
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
    #[validate_nested]
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
    let codecs = ValueCodecRegistry::empty();
    let graph = ModelResolver::new(ResolveInputs {
        models: &models,
        codecs: &codecs,
    })
    .resolve_structure()
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
    let codecs = ValueCodecRegistry::empty();
    let graph = ModelResolver::new(ResolveInputs {
        models: &models,
        codecs: &codecs,
    })
    .resolve_structure()
    .expect("structure");
    let validators = ValidatorRegistry::from_registrations([REGISTRATION]).expect("validator registry");
    let plan = ValidationPlan::build(
        metadata,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("binding")
    .with_model_rule(ModelRuleBinding::from_prepared::<TestModel>(
        ValidatorId::new("test.model.reject"),
        Arc::new(RejectModel),
    ));
    let report = plan
        .validate(
            ReflectedRef::new(&TestModel { name: "bad".to_owned() }),
            &ValidationOptions::default(),
        )
        .expect("execution");
    assert_eq!(report.violations().len(), 2);
    assert_eq!(report.violations()[0].path().render(), "");
}

#[test]
fn executes_element_selector_for_borrowed_slice() {
    let metadata = TypeMetadata::of::<SelectorFixture>();
    let reflection = ReflectRegistry::initialize().expect("reflection registry");
    let models = ModelRegistry::from_reflect_registry(reflection).expect("model registry");
    let codecs = ValueCodecRegistry::empty();
    let graph = ModelResolver::new(ResolveInputs {
        models: &models,
        codecs: &codecs,
    })
    .resolve_structure()
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
    let comparison_limited = ValidationOptions::default().with_max_comparisons(NonZeroUsize::new(1).expect("non-zero"));
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
    let codecs = ValueCodecRegistry::empty();
    let graph = ModelResolver::new(ResolveInputs {
        models: &models,
        codecs: &codecs,
    })
    .resolve_structure()
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
    let models = ModelRegistry::from_metadata(&[(metadata, source())], &[]).expect("model registry");
    let codecs = ValueCodecRegistry::empty();
    let graph = ModelResolver::new(ResolveInputs {
        models: &models,
        codecs: &codecs,
    })
    .resolve_structure()
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
    let depth = ValidationOptions::default().with_max_depth(NonZeroUsize::new(1).expect("non-zero"));
    assert!(plan.validate(value.clone(), &depth).is_ok());
    let nodes = ValidationOptions::default().with_max_nodes(NonZeroUsize::new(1).expect("non-zero"));
    assert!(plan.validate(value, &nodes).is_err());
}
