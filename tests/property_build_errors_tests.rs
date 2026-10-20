// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Invalid generated property overlays retain ordered, actionable diagnostics.

use std::error::Error;

use qubit_model_derive::Model;
use qubit_model_metadata::__private::v8;
use qubit_model_metadata::metadata::GetterMetadata;
use qubit_model_metadata::metadata::GetterOutputKind;
use qubit_model_metadata::metadata::PropertyAccessError;
use qubit_model_metadata::metadata::PropertyBuildErrorKind;
use qubit_model_metadata::metadata::PropertySetFailure;
use qubit_model_metadata::metadata::PropertyValue;
use qubit_model_metadata::metadata::SetterMetadata;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_reflect::__private::codegen_v4::expression::TypeExpressionData;
use qubit_reflect::ReflectionContext;
use qubit_reflect::value::DynamicMut;
use qubit_reflect::value::DynamicOwned;
use qubit_reflect::value::DynamicRef;
use qubit_reflect::value::Local;

#[Model]
struct Record {
    name: String,
    count: u32,
}

#[Model]
struct Other {
    name: String,
}

#[cfg(feature = "generic")]
#[Model]
struct GenericRecord<T> {
    value: T,
}

fn context() -> ReflectionContext {
    ReflectionContext::builder()
        .include_type::<Record>()
        .include_type::<Other>()
        .include_type::<String>()
        .include_type::<u32>()
        .build()
        .expect("property fixture context")
}

fn expression(field: &qubit_model_metadata::metadata::FieldMetadata) -> &'static TypeExpressionData {
    field.type_expression_data().expect("generated field expression")
}

fn string_type_id() -> std::any::TypeId {
    std::any::TypeId::of::<String>()
}
fn u32_type_id() -> std::any::TypeId {
    std::any::TypeId::of::<u32>()
}

/// A malformed concrete overlay must not turn a fallible Property check into
/// a panic merely because the referenced field is definition-only metadata.
#[cfg(feature = "generic")]
#[test]
fn test_definition_only_property_field_returns_a_structured_error() {
    let definition = TypeMetadata::of::<GenericRecord<String>>()
        .generic_definition()
        .expect("generic definition");
    let fields = definition.fields();
    let model = TypeMetadata::of::<Record>();
    let property = v8::property_metadata("value", Some(expression(&fields[0])), Some(&fields[0]), None, None);
    let context = context();
    let errors = model
        .validate_properties_in(&context, &[property])
        .expect_err("definition-only field");
    assert_eq!(errors.errors().len(), 1);
    assert_eq!(errors.errors()[0].kind(), PropertyBuildErrorKind::ForeignField);
    assert_eq!(errors.errors()[0].property_name(), "value");
}

/// Sorting uses property name and then category, independent of input order.
#[test]
fn test_property_errors_preserve_all_failures_in_deterministic_order() {
    let model = TypeMetadata::of::<Record>();
    let context = context();
    let name = &model.fields()[0];
    let count = &model.fields()[1];
    let foreign = &TypeMetadata::of::<Other>().fields()[0];
    let properties = [
        v8::property_metadata("z_missing", Some(expression(name)), None, None, None),
        v8::property_metadata("a_type", Some(expression(count)), Some(name), None, None),
        v8::property_metadata("m_foreign", Some(expression(name)), Some(foreign), None, None),
        v8::property_metadata("a_type", Some(expression(count)), Some(count), None, None),
    ];
    let errors = model
        .validate_properties_in(&context, &properties)
        .expect_err("invalid property overlays");
    let details: Vec<_> = errors
        .errors()
        .iter()
        .map(|error| (error.property_name(), error.kind()))
        .collect();
    assert_eq!(
        details,
        [
            ("a_type", PropertyBuildErrorKind::InvalidName),
            ("a_type", PropertyBuildErrorKind::FieldTypeMismatch),
            ("m_foreign", PropertyBuildErrorKind::ForeignField),
            ("z_missing", PropertyBuildErrorKind::MissingSource),
        ]
    );
    assert!(errors.errors()[0].conflict().is_none());
    assert_eq!(
        errors.to_string(),
        concat!(
            "property InvalidName for `a_type`\n",
            "property FieldTypeMismatch for `a_type`\n",
            "property ForeignField for `m_foreign`\n",
            "property MissingSource for `z_missing`",
        )
    );
    assert!(errors.errors().iter().all(|error| error.conflict().is_none()));
    assert!(errors.source().is_none());
}

/// An empty name and missing source are independent failures at one location.
#[test]
fn test_empty_property_retains_both_name_and_source_diagnostics() {
    let model = TypeMetadata::of::<Record>();
    let context = context();
    let property = v8::property_metadata("", Some(expression(&model.fields()[0])), None, None, None);
    let errors = model
        .validate_properties_in(&context, &[property])
        .expect_err("empty property");
    assert_eq!(errors.errors().len(), 2);
    assert_eq!(errors.errors()[0].kind(), PropertyBuildErrorKind::InvalidName);
    assert_eq!(errors.errors()[1].kind(), PropertyBuildErrorKind::MissingSource);
    assert_eq!(errors.errors()[0].property_name(), "");
}

/// Static validation must never execute a getter to discover its signature.
fn never_get(_: DynamicRef<'_, Local>) -> Result<PropertyValue<'_>, PropertyAccessError> {
    panic!("metadata validation invoked the getter");
}

/// Static validation must never execute a setter to discover its signature.
fn never_set(_: DynamicMut<'_, Local>, _: DynamicOwned<Local>) -> Result<(), PropertySetFailure> {
    panic!("metadata validation invoked the setter");
}

/// Exact accessor owners and value types are checked without invoking adapters.
#[test]
fn test_accessor_signature_errors_are_aggregated_before_invocation() {
    let model = TypeMetadata::of::<Record>();
    let context = context();
    let name_type = expression(&model.fields()[0]);
    let count_type = expression(&model.fields()[1]);
    let wrong_getter_owner = v8::leak(GetterMetadata::new::<Other>(
        "get",
        name_type,
        string_type_id,
        GetterOutputKind::Borrowed,
        never_get,
    ));
    let wrong_getter_output = v8::leak(GetterMetadata::new::<Record>(
        "get",
        count_type,
        u32_type_id,
        GetterOutputKind::Borrowed,
        never_get,
    ));
    let wrong_setter_owner = v8::leak(SetterMetadata::new::<Other, String>("set", name_type, never_set));
    let wrong_setter_input = v8::leak(SetterMetadata::new::<Record, u32>("set", count_type, never_set));
    let properties = [
        v8::property_metadata("getter_owner", Some(name_type), None, Some(wrong_getter_owner), None),
        v8::property_metadata("getter_output", Some(name_type), None, Some(wrong_getter_output), None),
        v8::property_metadata("setter_owner", Some(name_type), None, None, Some(wrong_setter_owner)),
        v8::property_metadata("setter_input", Some(name_type), None, None, Some(wrong_setter_input)),
    ];
    let errors = model
        .validate_properties_in(&context, &properties)
        .expect_err("accessor signatures");
    let details: Vec<_> = errors
        .errors()
        .iter()
        .map(|error| (error.property_name(), error.kind()))
        .collect();
    assert_eq!(
        details,
        [
            ("getter_output", PropertyBuildErrorKind::GetterTypeMismatch),
            ("getter_owner", PropertyBuildErrorKind::GetterTypeMismatch),
            ("setter_input", PropertyBuildErrorKind::SetterTypeMismatch),
            ("setter_owner", PropertyBuildErrorKind::SetterTypeMismatch),
        ]
    );
}
