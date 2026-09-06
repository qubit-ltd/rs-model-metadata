#![cfg(feature = "validation")]

//! Focused model validation execution tests.

use std::sync::Arc;

use qubit_codec::ValueCodecRegistry;
use qubit_model_derive::Model;
use qubit_model_metadata::FragmentIdentity;
use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::ModelResolver;
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
