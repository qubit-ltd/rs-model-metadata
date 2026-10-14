// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Derive declarations deliver lossless configuration to the runtime preparer.

use std::sync::Arc;

use qubit_model_derive::Model;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::value::DynamicRef;
use qubit_reflect::value::Local;
use qubit_validator::ArgumentReader;
use qubit_validator::BindError;
use qubit_validator::BoundValidationContext;
use qubit_validator::DependencySpec;
use qubit_validator::ExecutionError;
use qubit_validator::InputType;
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

#[Model]
struct ConfiguredToken {
    #[validator(
        id = "arguments.allow_token",
        params(enabled = true, minimum = 7, accepted = "accepted")
    )]
    token: String,
}

/// Owns the selected declaration parameter after the temporary bind input ends.
struct AllowToken(String);

impl PreparedValidator for AllowToken {
    fn input_type(&self) -> InputType {
        InputType::Text
    }
    fn dependency_specs(&self) -> &'static [DependencySpec] {
        &[]
    }
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        if value.as_text() == Some(self.0.as_str()) {
            Ok(PreparedOutcome::Valid)
        } else {
            Ok(PreparedOutcome::Invalid(vec![ViolationDraft::new(ViolationCode::new(
                "unexpected_token",
            ))]))
        }
    }
}

/// Checks the typed consumer parameter contract before compiling the rule.
fn prepare(arguments: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
    assert_eq!(arguments.optional_bool("enabled")?, Some(true));
    assert_eq!(arguments.required_u32("minimum")?, 7);
    let accepted = arguments.required_str("accepted")?;
    arguments.finish()?;
    Ok(Arc::new(AllowToken(accepted.to_owned())))
}

static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare)];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
static REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("arguments.allow_token"),
    &DESCRIPTOR,
    RegistrationSource::new("argument-tests", "declaration", file!(), line!()),
);

/// Typed parameters reach preparation, then influence real plan execution.
#[test]
fn test_all_argument_kinds_survive_declaration_binding_and_execution() {
    let models = ModelRegistry::from_static_metadata(&[]).expect("isolated model registry");
    let root = TypeMetadata::of::<ConfiguredToken>();
    let roots = [root];
    let graph = Arc::new(
        StructureResolver::new(ResolveInputs {
            models: &models,
            roots: &roots,
        })
        .resolve()
        .expect("valid model structure"),
    );
    let validators = ValidatorRegistry::from_registrations([REGISTRATION]).expect("registered rule");
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: Arc::clone(&graph),
            validators: &validators,
        },
    )
    .expect("all typed parameters accepted");
    assert_eq!(plan.binding_count(), 1);
    for (token, expected_failures) in [("accepted", 0), ("rejected", 1)] {
        let value = ConfiguredToken {
            token: token.to_owned(),
        };
        let report = plan
            .validate(DynamicRef::<Local>::new(&value), &ValidationOptions::default())
            .expect("configured rule executes");
        assert_eq!(report.violations().len(), expected_failures);
        assert!(!report.is_truncated());
    }
}
