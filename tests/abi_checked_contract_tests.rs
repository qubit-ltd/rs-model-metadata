// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Regression tests for the checked generated-code v8 ABI.

use std::any::TypeId;

#[cfg(feature = "generic")]
use qubit_model_derive::Enum;
use qubit_model_metadata::__private::ModelTypeSeal;
use qubit_model_metadata::__private::TypeMetadataProvider;
use qubit_model_metadata::__private::v8;
use qubit_model_metadata::metadata::ConstraintMetadata;
use qubit_model_metadata::metadata::FieldAttributeMetadata;
use qubit_model_metadata::metadata::FieldMetadata;
use qubit_model_metadata::metadata::GetterMetadata;
use qubit_model_metadata::metadata::GetterOutputKind;
use qubit_model_metadata::metadata::IndexingReasons;
use qubit_model_metadata::metadata::PropertyAccessError;
use qubit_model_metadata::metadata::PropertyValue;
use qubit_model_metadata::metadata::SelectorMetadata;
use qubit_model_metadata::metadata::SelectorPosition;
use qubit_model_metadata::metadata::SequenceConstraint;
use qubit_model_metadata::metadata::SerdeFieldMetadata;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::registry::ModelRegistryErrorKind;
use qubit_model_metadata::registry::ModelRegistryOrigin;
use qubit_reflect::__private::codegen_v4::expression::ExpressionData;
use qubit_reflect::__private::codegen_v4::expression::TypeExpressionData;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::Reflect as ReflectTrait;
use qubit_reflect::descriptor::Visibility;
use qubit_reflect::error::TypeMismatch;
use qubit_reflect::value::DynamicRef;
use qubit_reflect::value::Local;

fn unavailable_identifier<'a>(value: DynamicRef<'a, Local>) -> Result<&'a qubit_id::Id, TypeMismatch> {
    Err(value.downcast::<qubit_id::Id>().unwrap_err().into_parts().0)
}

#[Reflect]
struct OneField {
    value: String,
}

#[Reflect]
struct TwoFields {
    first: String,
    second: String,
}

#[Reflect]
struct WrongTarget;

impl ModelTypeSeal for WrongTarget {}

impl TypeMetadataProvider for WrongTarget {
    fn __type_metadata() -> &'static TypeMetadata {
        static METADATA: std::sync::OnceLock<TypeMetadata> = std::sync::OnceLock::new();
        METADATA.get_or_init(|| {
            let role = v8::leak(v8::model_role());
            v8::GeneratedTypeMetadataBuilder::new::<OneField>(None, &[], role).finish_unchecked()
        })
    }
}

#[Reflect]
enum OneVariant {
    Ready,
}

fn unavailable_getter(_: DynamicRef<'_, Local>) -> Result<PropertyValue<'_>, PropertyAccessError> {
    Err(PropertyAccessError::AdapterUnavailable)
}

static STRING_EXPRESSION: TypeExpressionData = ExpressionData::Named {
    path: "String",
    definition: None,
    arguments: &[],
};

fn reflected_fields<T: ReflectTrait + 'static>() -> &'static [FieldMetadata] {
    let descriptor = TypeDescriptor::of_in::<T>(ReflectionContext::global());
    v8::leak_slice(
        descriptor
            .fields()
            .iter()
            .map(|field| {
                v8::field_metadata(
                    TypeId::of::<T>(),
                    field.declaring_variant().map(|variant| variant.index()),
                    field.index(),
                    Some(field.name()),
                    field.visibility(),
                    &STRING_EXPRESSION,
                    &[],
                    &[],
                    &[],
                    &SerdeFieldMetadata::DEFAULT,
                )
            })
            .collect(),
    )
}

fn assert_abi_error(metadata: &'static TypeMetadata, expected_code: &str) {
    let error = ModelRegistry::from_static_metadata(ReflectionContext::global(), &[(metadata, source())])
        .expect_err("invalid metadata must fail context-bound ABI validation");
    assert_eq!(error.kind(), ModelRegistryErrorKind::InvalidMetadata);
    assert_eq!(error.abi_cause().expect("ABI cause").code(), expected_code);
}

fn source() -> &'static ModelRegistryOrigin {
    static SOURCE: std::sync::OnceLock<ModelRegistryOrigin> = std::sync::OnceLock::new();
    SOURCE.get_or_init(|| ModelRegistryOrigin {
        crate_name: "abi-checked-contract-tests",
        crate_version: None,
        module_path: module_path!(),
        file: Some(file!()),
        line: Some(line!()),
        column: Some(1),
    })
}

#[test]
fn test_context_projection_rejects_missing_field_overlay() {
    let role = v8::leak(v8::model_role());
    let missing_field = v8::leak(v8::GeneratedTypeMetadataBuilder::new::<OneField>(None, &[], role).finish_unchecked());
    assert_abi_error(missing_field, "QMM-ABI-003");
}

#[test]
fn test_try_of_returns_structured_abi_violation_without_panicking() {
    let error = TypeMetadata::try_of::<WrongTarget>().expect_err("wrong descriptor must fail");
    assert_eq!(error.code(), "QMM-ABI-001");
}

#[test]
fn test_finish_rejects_duplicate_properties_and_wrong_getter_targets() {
    let fields = reflected_fields::<OneField>();
    let role = v8::leak(v8::model_role());
    let expression = fields[0].type_expression_data().expect("field expression");
    let property = v8::property_metadata("value", Some(expression), Some(&fields[0]), None, None);
    let properties = v8::leak_slice(vec![property, property]);
    let duplicate = v8::leak(
        v8::GeneratedTypeMetadataBuilder::new::<OneField>(None, fields, role)
            .properties(properties)
            .finish_unchecked(),
    );
    assert_abi_error(duplicate, "QMM-ABI-004");

    let getter = v8::leak(GetterMetadata::new::<WrongTarget>(
        "value",
        expression,
        TypeId::of::<String>,
        GetterOutputKind::Borrowed,
        unavailable_getter,
    ));
    let properties = v8::leak_slice(vec![v8::property_metadata(
        "value",
        Some(expression),
        Some(&fields[0]),
        Some(getter),
        None,
    )]);
    let wrong_target = v8::leak(
        v8::GeneratedTypeMetadataBuilder::new::<OneField>(None, fields, role)
            .properties(properties)
            .finish_unchecked(),
    );
    assert_abi_error(wrong_target, "QMM-ABI-004");
}

#[test]
fn test_finish_rejects_invalid_role_payloads() {
    let one_fields = reflected_fields::<OneField>();
    let entity_role = v8::leak(v8::entity_role(&one_fields[0], unavailable_identifier));
    let invalid_identifier =
        v8::leak(v8::GeneratedTypeMetadataBuilder::new::<OneField>(None, one_fields, entity_role).finish_unchecked());
    assert_abi_error(invalid_identifier, "QMM-ABI-010");

    let two_fields = reflected_fields::<TwoFields>();
    let value_role = v8::leak(v8::value_role(Some(&two_fields[0]), None));
    let invalid_transparent =
        v8::leak(v8::GeneratedTypeMetadataBuilder::new::<TwoFields>(None, two_fields, value_role).finish_unchecked());
    assert_abi_error(invalid_transparent, "QMM-ABI-011");

    let enum_role = v8::leak(v8::enum_role(&[]));
    let invalid_enum =
        v8::leak(v8::GeneratedTypeMetadataBuilder::new::<OneVariant>(None, &[], enum_role).finish_unchecked());
    assert_abi_error(invalid_enum, "QMM-ABI-012");
}

#[test]
fn test_finish_rejects_selector_on_incompatible_field_shape() {
    let descriptor = TypeDescriptor::of::<OneField>();
    let selector = v8::leak(SelectorMetadata::new(SelectorPosition::Element, &[], &[], None, None));
    let constraints = v8::leak_slice(vec![ConstraintMetadata::Sequence(
        SequenceConstraint::new(None, None, false).with_element(selector),
    )]);
    let attributes = v8::leak_slice(vec![FieldAttributeMetadata::Constraint(&constraints[0])]);
    let fields = v8::leak_slice(vec![v8::field_metadata(
        descriptor.type_id(),
        None,
        0,
        Some("value"),
        Visibility::Private,
        &STRING_EXPRESSION,
        attributes,
        constraints,
        &[],
        &SerdeFieldMetadata::DEFAULT,
    )]);
    let role = v8::leak(v8::model_role());

    let mismatch = v8::leak(v8::GeneratedTypeMetadataBuilder::new::<OneField>(None, fields, role).finish_unchecked());
    assert_abi_error(mismatch, "QMM-ABI-020");
}

/// Checked construction rejects a field whose declared owner is another type.
#[test]
fn test_finish_rejects_wrong_field_owner_identity() {
    let descriptor = TypeDescriptor::of::<OneField>();
    let reflected = descriptor.fields().get_at(0).expect("value field");
    let fields = v8::leak_slice(vec![FieldMetadata::from_reflect(
        std::any::TypeId::of::<TwoFields>(),
        reflected,
    )]);
    let role = v8::leak(v8::model_role());
    let invalid_owner =
        v8::leak(v8::GeneratedTypeMetadataBuilder::new::<OneField>(None, fields, role).finish_unchecked());
    let error = ModelRegistry::from_static_metadata(ReflectionContext::global(), &[(invalid_owner, source())])
        .expect_err("wrong field owner identity");
    assert_eq!(error.abi_cause().expect("ABI cause").code(), "QMM-ABI-003");
    assert!(error.to_string().contains("identity"));
}

/// Checked ABI validation must reject large duplicate lists without a numeric
/// overflow panic or a wrapped counter accepting the malformed declaration.
#[test]
fn test_singleton_semantic_counts_cannot_overflow() {
    let descriptor = TypeDescriptor::of::<OneField>();
    let role = v8::leak(v8::model_role());
    for count in [2, 255, 256, 257] {
        let attributes = v8::leak_slice(vec![FieldAttributeMetadata::Indexed(IndexingReasons::EXPLICIT); count]);
        let fields = v8::leak_slice(vec![v8::field_metadata(
            descriptor.type_id(),
            None,
            0,
            Some("value"),
            Visibility::Private,
            &STRING_EXPRESSION,
            attributes,
            &[],
            &[],
            &SerdeFieldMetadata::DEFAULT,
        )]);
        let metadata =
            v8::leak(v8::GeneratedTypeMetadataBuilder::new::<OneField>(None, fields, role).finish_unchecked());
        assert_abi_error(metadata, "QMM-ABI-015");
    }
}

#[cfg(feature = "generic")]
#[Enum]
enum GenericVariant<T> {
    Value(T),
}

/// A source-level variant is not a concrete overlay, even when counts and
/// indices happen to match the requested concrete enum.
#[cfg(feature = "generic")]
#[test]
fn test_concrete_enum_rejects_definition_only_variants_without_panicking() {
    let definition = TypeMetadata::of::<GenericVariant<String>>()
        .generic_definition()
        .expect("generic definition retained");
    let role = v8::leak(v8::enum_role(definition.variants()));
    let metadata = v8::leak(v8::GeneratedTypeMetadataBuilder::new::<OneVariant>(None, &[], role).finish_unchecked());
    assert_abi_error(metadata, "QMM-ABI-013");
}
