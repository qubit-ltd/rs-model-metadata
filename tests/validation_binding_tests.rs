//! Binding-only validation plan tests.

use std::sync::Arc;

use qubit_codec::ValueCodecRegistry;
use qubit_model_derive::Model;
use qubit_model_metadata::FragmentIdentity;
use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::ModelResolver;
use qubit_model_metadata::ResolveInputs;
use qubit_model_metadata::TypeMetadata;
use qubit_model_metadata::ValidationBuildInputs;
use qubit_model_metadata::ValidationPlan;
use qubit_validator::BindError;
use qubit_validator::BoundValidationContext;
use qubit_validator::ExecutionError;
use qubit_validator::InputType;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::RuleOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorSignature;
use qubit_validator::ValidatorRegistry;

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

fn prepare_text(_: &[qubit_validator::NamedValidationArgument<'_>])
    -> Result<Arc<dyn PreparedValidator>, BindError>
{
    Ok(Arc::new(AcceptText))
}

static TEXT_SIGNATURES: &[ValidatorSignature] = &[
    ValidatorSignature::new(InputType::Text, &[], prepare_text),
];
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

fn inputs<'a>(
    models: &'a ModelRegistry<'a>,
    codecs: &'a ValueCodecRegistry,
) -> ResolveInputs<'a> {
    ResolveInputs {
        models,
        codecs,
    }
}

#[test]
fn structure_resolution_and_binding_are_separate() {
    let owner = TypeMetadata::of::<Owner>();
    let fixture = TypeMetadata::of::<BindingFixture>();
    let models = ModelRegistry::from_metadata(&[(owner, source()), (fixture, source())], &[])
        .expect("isolated model registry");
    let validators = ValidatorRegistry::from_registrations([TEXT_REGISTRATION])
        .expect("isolated validator registry");
    let codecs = ValueCodecRegistry::empty();
    let graph = ModelResolver::new(inputs(&models, &codecs))
        .resolve_structure()
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
