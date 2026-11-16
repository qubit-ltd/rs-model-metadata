// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! A failed property path must not suppress independent relationship failures.

use std::any::TypeId;
use std::sync::OnceLock;

use qubit_model_derive::Model;
use qubit_model_metadata::__private::ModelMetadataCapability;
use qubit_model_metadata::__private::v8;
use qubit_model_metadata::metadata::DeclaredEntityTarget;
use qubit_model_metadata::metadata::FieldAttributeMetadata;
use qubit_model_metadata::metadata::FieldReferenceMetadata;
use qubit_model_metadata::metadata::ModelId;
use qubit_model_metadata::metadata::NavigationStep;
use qubit_model_metadata::metadata::ObjectPath;
use qubit_model_metadata::metadata::ReferenceSelection;
use qubit_model_metadata::metadata::SerdeFieldMetadata;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::registry::ModelRegistryOrigin;
use qubit_model_metadata::resolve::ResolveErrorKind;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_reflect::__private::codegen_v4::expression::ExpressionData;
use qubit_reflect::__private::codegen_v4::expression::GenericExpressionArgument;
use qubit_reflect::__private::codegen_v4::expression::TypeExpressionData;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectionContext;
use qubit_reflect::capability::CapabilityRegistration;
use qubit_reflect::descriptor::Visibility;

#[Model(id = "error.WrongMetadata")]
struct WrongMetadataFixture {
    value: String,
}

fn wrong_metadata() -> &'static TypeMetadata {
    TypeMetadata::of::<WrongMetadataFixture>()
}

static INVALID_EXPRESSION: TypeExpressionData = ExpressionData::Named {
    path: "Invalid",
    definition: None,
    arguments: &[GenericExpressionArgument::Const(
        qubit_reflect::__private::codegen_v4::expression::ConstExpression::UnsignedInteger(1),
    )],
};
static U64_EXPRESSION: TypeExpressionData = ExpressionData::Named {
    path: "u64",
    definition: None,
    arguments: &[],
};

#[Reflect]
struct Invalid<const N: usize>;

#[Reflect]
struct Root {
    invalid: Invalid<1>,
    target: u64,
}

fn reflection() -> ReflectionContext {
    ReflectionContext::builder()
        .include_type::<Root>()
        .include_type::<Invalid<1>>()
        .include_type::<u64>()
        .include_type::<WrongMetadataFixture>()
        .include_type::<String>()
        .register(CapabilityRegistration::for_type::<Invalid<1>, ModelMetadataCapability>(
            wrong_metadata,
        ))
        .build()
        .expect("the typed provider is valid until its metadata is checked against Invalid")
}

fn metadata() -> &'static TypeMetadata {
    static METADATA: OnceLock<TypeMetadata> = OnceLock::new();
    METADATA.get_or_init(|| {
        let reference = v8::leak(FieldReferenceMetadata::new(
            v8::leak(DeclaredEntityTarget::ModelId(ModelId::new("missing.Target"))),
            v8::leak(ReferenceSelection::Entity),
            false,
            Some(v8::leak(
                ObjectPath::new(&[NavigationStep::Property("invalid"), NavigationStep::Property("value")]).unwrap(),
            )),
        ));
        let fields = v8::leak_slice(vec![
            v8::field_metadata(
                TypeId::of::<Root>(),
                None,
                0,
                Some("invalid"),
                Visibility::Private,
                &INVALID_EXPRESSION,
                &[],
                &[],
                &[],
                &SerdeFieldMetadata::DEFAULT,
            ),
            v8::field_metadata(
                TypeId::of::<Root>(),
                None,
                1,
                Some("target"),
                Visibility::Private,
                &U64_EXPRESSION,
                v8::leak_slice(vec![FieldAttributeMetadata::Reference(reference)]),
                &[],
                &[],
                &SerdeFieldMetadata::DEFAULT,
            ),
        ]);
        let properties = v8::leak_slice(
            fields
                .iter()
                .zip([&INVALID_EXPRESSION, &U64_EXPRESSION])
                .map(|(field, expression)| {
                    v8::property_metadata(field.name().unwrap(), Some(expression), Some(field), None, None)
                })
                .collect(),
        );
        v8::GeneratedTypeMetadataBuilder::new::<Root>(
            Some(ModelId::new("error.PathRoot")),
            fields,
            v8::leak(v8::model_role()),
        )
        .properties(properties)
        .finish_unchecked()
    })
}

fn source() -> &'static ModelRegistryOrigin {
    static SOURCE: OnceLock<ModelRegistryOrigin> = OnceLock::new();
    SOURCE.get_or_init(|| ModelRegistryOrigin {
        crate_name: "metadata-path-tests",
        crate_version: None,
        module_path: module_path!(),
        file: Some(file!()),
        line: Some(line!()),
        column: Some(1),
    })
}

#[test]
fn test_path_conflict_and_missing_target_are_both_reported() {
    let context = reflection();
    let models = ModelRegistry::from_static_metadata(&context, &[(metadata(), source())]).unwrap();
    let errors = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: &models,
    })
    .resolve()
    .unwrap_err();

    assert_eq!(errors.errors().len(), 3);
    let cause = errors
        .errors()
        .iter()
        .find(|error| error.kind() == ResolveErrorKind::MetadataResolution && error.path().is_some())
        .unwrap();
    assert_eq!(cause.model_id(), Some("error.PathRoot"));
    assert_eq!(cause.path().unwrap().to_string(), "target");
    assert_eq!(cause.object_path().unwrap().to_string(), "invalid/value");
    assert!(cause.cause().is_some());
    assert!(!cause.sources().is_empty());
    assert!(
        errors
            .errors()
            .iter()
            .any(|error| error.kind() == ResolveErrorKind::MissingModelId)
    );
    assert!(
        !errors
            .errors()
            .iter()
            .any(|error| error.kind() == ResolveErrorKind::MissingProperty)
    );
}
