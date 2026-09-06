//! Focused model validation execution tests.

use std::sync::Arc;

use qubit_codec::ValueCodecRegistry;
use qubit_model_derive::Model;
use qubit_model_metadata::{FragmentIdentity, ModelRegistry, ModelResolver, ResolveInputs, TypeMetadata, ValidationBuildInputs, ValidationOptions, ValidationPlan};
use qubit_validator::{BindError, BoundValidationContext, ExecutionError, PreparedValidator, RuleOutcome, ValidationValue, ValidatorDescriptor, ValidatorRegistration, ValidatorSignature, Violation, ViolationCode};
use qubit_validator::{RegistrationSource, ValidatorId, ValidatorRegistry};

#[Model(id = "validation.TestModel")]
struct TestModel {
    #[validator(id = "test.reject")]
    name: String,
}

struct Reject;
impl PreparedValidator for Reject {
    fn validate(&self, value: ValidationValue<'_>, _: &BoundValidationContext<'_>) -> Result<RuleOutcome, ExecutionError> {
        assert!(value.as_text().is_some());
        Ok(RuleOutcome::Invalid(vec![Violation::new(ValidatorId::new("test.reject"), ViolationCode::new("value.invalid"))]))
    }
}
fn prepare(_: &[qubit_validator::NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> { Ok(Arc::new(Reject)) }
static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(qubit_validator::InputType::Text, &[], prepare)];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
static REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(ValidatorId::new("test.reject"), &DESCRIPTOR, RegistrationSource::new("model-validation-tests", "test", file!(), line!()));

fn source() -> &'static FragmentIdentity {
    Box::leak(Box::new(FragmentIdentity::new("model-validation-tests", "test", line!(), 1, "test", 1)))
}

#[test]
fn executes_bound_rule_and_prefixes_field_path() {
    let metadata = TypeMetadata::of::<TestModel>();
    let models = ModelRegistry::from_metadata(&[(metadata, source())], &[]).expect("model registry");
    let codecs = ValueCodecRegistry::empty();
    let graph = ModelResolver::new(ResolveInputs { models: &models, codecs: &codecs }).resolve_structure().expect("structure");
    let validators = ValidatorRegistry::from_registrations([REGISTRATION]).expect("validator registry");
    let plan = ValidationPlan::build(metadata, ValidationBuildInputs { graph: &graph, validators: &validators }).expect("binding");
    let report = plan.validate(qubit_model_metadata::ReflectedRef::new(&TestModel { name: "bad".to_owned() }), &ValidationOptions::default()).expect("execution");
    assert!(!report.is_valid());
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.violations()[0].path().render(), "name");
}
