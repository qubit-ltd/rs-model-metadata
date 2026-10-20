// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![cfg(feature = "validation")]

//! Transparent value field validation through public plan execution.

use std::sync::Arc;

use qubit_model_derive::Model;
use qubit_model_derive::Value;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildErrorKind;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::ReflectionContext;
use qubit_reflect::value::DynamicRef;
use qubit_reflect::value::Local;
use qubit_validator::SkipReason;
use qubit_validator::ValidatorRegistry;

#[Value(transparent)]
struct Email(#[text(min_chars = 1)] String);

#[Value(transparent)]
struct OptionalEmail(#[text(min_chars = 1)] Option<String>);

#[cfg(feature = "generic")]
#[Value(transparent)]
#[derive(Default)]
struct WrappedText(#[text(min_chars = 1)] String);

#[cfg(feature = "generic")]
#[Model(
    no_redact,
    no_serialize,
    no_deserialize,
    id = "qubit_model_metadata.validation.GenericOptionalWrapper"
)]
struct GenericOptionalWrapper<T> {
    value: Option<T>,
}

#[cfg(feature = "generic")]
#[Model(no_redact, no_serialize, no_deserialize)]
struct GenericWrapperEnvelope {
    wrapped: GenericOptionalWrapper<WrappedText>,
}

#[Model]
struct Envelope {
    email: Option<Email>,
}

#[Model]
struct TupleContainer {
    pair: (Email, String),
}

#[test]
fn test_transparent_value_nested_field_reports_violation_and_optional_skip() {
    let root = TypeMetadata::of::<Envelope>();
    let roots = [root];
    let models = ModelRegistry::from_context(ReflectionContext::global()).expect("model registry");
    let graph = Arc::new(
        StructureResolver::new(ResolveInputs {
            models: &models,
            roots: &roots,
        })
        .resolve()
        .expect("envelope structure"),
    );
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph,
            validators: &validators,
        },
    )
    .expect("transparent field must bind");

    let invalid = Envelope {
        email: Some(Email(String::new())),
    };
    let report = plan
        .validate_dynamic(DynamicRef::<Local>::new(&invalid), &ValidationOptions::default())
        .expect("transparent field execution");
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.violations()[0].path().render(), "email[0]");
    assert!(report.skipped().is_empty());

    let missing = Envelope { email: None };
    let report = plan
        .validate_dynamic(DynamicRef::<Local>::new(&missing), &ValidationOptions::default())
        .expect("missing email skips the field");
    assert!(report.violations().is_empty());
    assert_eq!(report.skipped().len(), 1);
    assert_eq!(report.skipped()[0].path().render(), "email[0]");
    assert_eq!(report.skipped()[0].reason(), SkipReason::MissingOptional);
}

#[test]
fn test_transparent_value_root_field_reports_numeric_path() {
    let root = TypeMetadata::of::<Email>();
    let roots = [root];
    let models = ModelRegistry::from_context(ReflectionContext::global()).expect("model registry");
    let graph = Arc::new(
        StructureResolver::new(ResolveInputs {
            models: &models,
            roots: &roots,
        })
        .resolve()
        .expect("email structure"),
    );
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph,
            validators: &validators,
        },
    )
    .expect("transparent root field must bind");
    let report = plan
        .validate_dynamic(
            DynamicRef::<Local>::new(&Email(String::new())),
            &ValidationOptions::default(),
        )
        .expect("transparent root execution");
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.violations()[0].path().render(), "[0]");
}

#[test]
fn test_transparent_value_optional_scalar_root_reports_violation_and_skip() {
    let root = TypeMetadata::of::<OptionalEmail>();
    let roots = [root];
    let models = ModelRegistry::from_context(ReflectionContext::global()).expect("model registry");
    let graph = Arc::new(
        StructureResolver::new(ResolveInputs {
            models: &models,
            roots: &roots,
        })
        .resolve()
        .expect("optional email structure"),
    );
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph,
            validators: &validators,
        },
    )
    .expect("transparent optional scalar must bind");

    let invalid = OptionalEmail(Some(String::new()));
    let report = plan
        .validate_dynamic(DynamicRef::<Local>::new(&invalid), &ValidationOptions::default())
        .expect("present optional scalar execution");
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.violations()[0].path().render(), "[0]");
    assert!(report.skipped().is_empty());

    let missing = OptionalEmail(None);
    let report = plan
        .validate_dynamic(DynamicRef::<Local>::new(&missing), &ValidationOptions::default())
        .expect("missing optional scalar skips the field");
    assert!(report.violations().is_empty());
    assert_eq!(report.skipped().len(), 1);
    assert_eq!(report.skipped()[0].path().render(), "[0]");
    assert_eq!(report.skipped()[0].reason(), SkipReason::MissingOptional);
}

#[cfg(feature = "generic")]
#[test]
fn test_transparent_value_validation_through_generic_optional_wrapper() {
    let root = TypeMetadata::of::<GenericWrapperEnvelope>();
    let roots = [root];
    let models = ModelRegistry::from_context(ReflectionContext::global()).expect("model registry");
    let graph = Arc::new(
        StructureResolver::new(ResolveInputs {
            models: &models,
            roots: &roots,
        })
        .resolve()
        .expect("generic wrapper structure"),
    );
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph,
            validators: &validators,
        },
    )
    .expect("transparent constraint must bind through generic wrapper");

    let present = GenericWrapperEnvelope {
        wrapped: GenericOptionalWrapper {
            value: Some(WrappedText(String::new())),
        },
    };
    let report = plan
        .validate_dynamic(DynamicRef::<Local>::new(&present), &ValidationOptions::default())
        .expect("present wrapped value execution");
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.violations()[0].path().render(), "wrapped.value[0]");

    let absent = GenericWrapperEnvelope {
        wrapped: GenericOptionalWrapper { value: None },
    };
    let report = plan
        .validate_dynamic(DynamicRef::<Local>::new(&absent), &ValidationOptions::default())
        .expect("absent wrapped value execution");
    assert!(report.violations().is_empty());
    assert_eq!(report.skipped().len(), 1);
    assert_eq!(report.skipped()[0].path().render(), "wrapped.value[0]");
    assert_eq!(report.skipped()[0].reason(), SkipReason::MissingOptional);
}

#[test]
fn test_ordinary_tuple_field_remains_unsupported() {
    let root = TypeMetadata::of::<TupleContainer>();
    let roots = [root];
    let models = ModelRegistry::from_context(ReflectionContext::global()).expect("model registry");
    let graph = Arc::new(
        StructureResolver::new(ResolveInputs {
            models: &models,
            roots: &roots,
        })
        .resolve()
        .expect("ordinary tuple structure"),
    );
    let validators = ValidatorRegistry::empty();
    let errors = match ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph,
            validators: &validators,
        },
    ) {
        Ok(_) => panic!("ordinary tuple validation must remain unsupported"),
        Err(errors) => errors,
    };
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
    assert_eq!(errors[0].path(), Some("pair.0.0"));
}
