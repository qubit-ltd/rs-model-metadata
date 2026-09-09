// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo fixture rather than an integration-test
// target.

//! Defines target-side models for linked registration and resolution fixtures.

use qubit_codec::ValueCodecDescriptor;
use qubit_codec::ValueCodecId;
use qubit_codec::ValueCodecRegistration;
use qubit_codec::ValueCodecRegistrationSource;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_model_derive::Entity;
#[cfg(feature = "duplicate-fixture")]
use qubit_model_derive::Model;
use qubit_model_metadata::__private::qubit_id::Id;
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
use qubit_validator::ValidatorSignature;

#[Entity(id = "test.linked.Target")]
pub struct Target {
    #[identifier]
    pub id: Id,
}

#[Model(id = "test.linked.Duplicate")]
#[cfg(feature = "duplicate-fixture")]
pub struct Duplicate;

/// A policy implementation in a different crate from its model declaration.
#[derive(Default)]
pub struct TextCodec;

impl ValueEncoder<String> for TextCodec {
    type Output = String;
    type Error = core::convert::Infallible;

    fn encode(&mut self, input: &String) -> Result<String, Self::Error> {
        Ok(input.clone())
    }
}

impl ValueDecoder<str> for TextCodec {
    type Output = String;
    type Error = core::convert::Infallible;

    fn decode(&mut self, input: &str) -> Result<String, Self::Error> {
        Ok(input.to_owned())
    }
}

static CODEC_DESCRIPTOR: ValueCodecDescriptor = ValueCodecDescriptor::of::<TextCodec, String>();
pub static CODEC: ValueCodecRegistration = ValueCodecRegistration::new(
    ValueCodecId::new("test.linked.text"),
    &CODEC_DESCRIPTOR,
    ValueCodecRegistrationSource::new("model-b", "models", file!(), line!()),
);

struct TextRule;

impl PreparedValidator for TextRule {
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<RuleOutcome, ExecutionError> {
        assert_eq!(value.as_text(), Some("cross-crate"));
        Ok(RuleOutcome::Valid)
    }
}

fn prepare_text(_: &[NamedValidationArgument<'_>]) -> Result<std::sync::Arc<dyn PreparedValidator>, BindError> {
    Ok(std::sync::Arc::new(TextRule))
}

static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare_text)];
static RULE_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
pub static RULE: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("test.linked.text"),
    &RULE_DESCRIPTOR,
    RegistrationSource::new("model-b", "models", file!(), line!()),
);
