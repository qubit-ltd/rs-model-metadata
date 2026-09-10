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
use qubit_reflect::ReflectedRef;
use qubit_validator::BindError;
use qubit_validator::BoundValidationContext;
use qubit_validator::ExecutionError;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::RuleOutcome;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorSignature;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;

#[Model]
struct ConfiguredToken {
    #[validator(
        id = "arguments.allow_token",
        params(
            enabled = true,
            lower = -170141183460469231731687303715884105728,
            upper = 340282366920938463463374607431768211455,
            accepted = "accepted",
            switches = [true, false],
            signed = [-170141183460469231731687303715884105728, 170141183460469231731687303715884105727],
            unsigned = [0, 340282366920938463463374607431768211455],
            labels = ["first", "second"]
        )
    )]
    token: String,
}

/// Owns the selected declaration parameter after the temporary bind input ends.
struct AllowToken(String);

impl PreparedValidator for AllowToken {
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<RuleOutcome, ExecutionError> {
        if value.as_text() == Some(self.0.as_str()) {
            Ok(RuleOutcome::Valid)
        } else {
            Ok(RuleOutcome::Invalid(vec![Violation::new(
                ValidatorId::new("arguments.allow_token"),
                ViolationCode::new("unexpected_token"),
            )]))
        }
    }
}

/// Checks the consumer-visible parameter contract before compiling the rule.
fn prepare(arguments: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let expected = [
        ("enabled", ValidationArgument::Bool(true)),
        ("lower", ValidationArgument::Integer(i128::MIN)),
        ("upper", ValidationArgument::Unsigned(u128::MAX)),
        ("accepted", ValidationArgument::String("accepted")),
        ("switches", ValidationArgument::BoolList(&[true, false])),
        ("signed", ValidationArgument::IntegerList(&[i128::MIN, i128::MAX])),
        ("unsigned", ValidationArgument::UnsignedList(&[0, u128::MAX])),
        ("labels", ValidationArgument::StringList(&["first", "second"])),
    ];
    let received: Vec<_> = arguments
        .iter()
        .map(|argument| (argument.name(), argument.value()))
        .collect();
    assert_eq!(
        received, expected,
        "all names, types, bounds, and list order must survive binding"
    );
    let ValidationArgument::String(accepted) = arguments[3].value() else {
        unreachable!("the complete public argument contract was checked above");
    };
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
    let models = ModelRegistry::from_metadata(&[]).expect("isolated model registry");
    let root = TypeMetadata::of::<ConfiguredToken>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("valid model structure");
    let validators = ValidatorRegistry::from_registrations([REGISTRATION]).expect("registered rule");
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
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
            .validate(ReflectedRef::new(&value), &ValidationOptions::default())
            .expect("configured rule executes");
        assert_eq!(report.violations().len(), expected_failures);
        assert!(!report.is_truncated());
    }
}
