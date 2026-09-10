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
use qubit_model_metadata::__private::v7;
use qubit_model_metadata::metadata::GetterMetadata;
use qubit_model_metadata::metadata::GetterOutputKind;
use qubit_model_metadata::metadata::PropertyAccessError;
use qubit_model_metadata::metadata::PropertyBuildErrorKind;
use qubit_model_metadata::metadata::PropertySetFailure;
use qubit_model_metadata::metadata::PropertyValue;
use qubit_model_metadata::metadata::SetterMetadata;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_reflect::ReflectedMut;
use qubit_reflect::ReflectedOwned;
use qubit_reflect::ReflectedRef;

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

/// A malformed concrete overlay must not turn a fallible Property check into
/// a panic merely because the referenced field is definition-only metadata.
#[cfg(feature = "generic")]
#[test]
fn test_definition_only_property_field_returns_a_structured_error() {
    let definition = TypeMetadata::of::<GenericRecord<String>>()
        .generic_definition()
        .expect("generic definition");
    let fields = definition.fields();
    let model = v7::GeneratedTypeMetadataBuilder::new(
        TypeMetadata::of::<Record>().descriptor(),
        None,
        fields,
        v7::leak(v7::model_role()),
    )
    .finish_unchecked();
    let property = v7::property_metadata("value", fields[0].type_ref(), Some(&fields[0]), None, None);
    let errors = model
        .validate_properties(&[property])
        .expect_err("definition-only field");
    assert_eq!(errors.errors().len(), 1);
    assert_eq!(errors.errors()[0].kind(), PropertyBuildErrorKind::ForeignField);
    assert_eq!(errors.errors()[0].property_name(), "value");
}

/// Sorting uses property name and then category, independent of input order.
#[test]
fn test_property_errors_preserve_all_failures_in_deterministic_order() {
    let model = TypeMetadata::of::<Record>();
    let name = &model.fields()[0];
    let count = &model.fields()[1];
    let foreign = &TypeMetadata::of::<Other>().fields()[0];
    let properties = [
        v7::property_metadata("z_missing", name.type_ref(), None, None, None),
        v7::property_metadata("a_type", count.type_ref(), Some(name), None, None),
        v7::property_metadata("m_foreign", name.type_ref(), Some(foreign), None, None),
        v7::property_metadata("a_type", count.type_ref(), Some(count), None, None),
    ];
    let errors = model
        .validate_properties(&properties)
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
    assert_eq!(
        errors.to_string(),
        concat!(
            "property InvalidName for `a_type`\n",
            "property FieldTypeMismatch for `a_type`\n",
            "property ForeignField for `m_foreign`\n",
            "property MissingSource for `z_missing`",
        )
    );
    assert!(errors.source().is_none());
}

/// An empty name and missing source are independent failures at one location.
#[test]
fn test_empty_property_retains_both_name_and_source_diagnostics() {
    let model = TypeMetadata::of::<Record>();
    let property = v7::property_metadata("", model.fields()[0].type_ref(), None, None, None);
    let errors = model.validate_properties(&[property]).expect_err("empty property");
    assert_eq!(errors.errors().len(), 2);
    assert_eq!(errors.errors()[0].kind(), PropertyBuildErrorKind::InvalidName);
    assert_eq!(errors.errors()[1].kind(), PropertyBuildErrorKind::MissingSource);
    assert_eq!(errors.errors()[0].property_name(), "");
}

/// Static validation must never execute a getter to discover its signature.
fn never_get(_: ReflectedRef<'_>) -> Result<PropertyValue<'_>, PropertyAccessError> {
    panic!("metadata validation invoked the getter");
}

/// Static validation must never execute a setter to discover its signature.
fn never_set(_: ReflectedMut<'_>, _: ReflectedOwned) -> Result<(), PropertySetFailure> {
    panic!("metadata validation invoked the setter");
}

/// Exact accessor owners and value types are checked without invoking adapters.
#[test]
fn test_accessor_signature_errors_are_aggregated_before_invocation() {
    let model = TypeMetadata::of::<Record>();
    let name_type = model.fields()[0].type_ref();
    let count_type = model.fields()[1].type_ref();
    let wrong_getter_owner = v7::leak(GetterMetadata::new::<Other>(
        "get",
        name_type,
        GetterOutputKind::Borrowed,
        never_get,
    ));
    let wrong_getter_output = v7::leak(GetterMetadata::new::<Record>(
        "get",
        count_type,
        GetterOutputKind::Borrowed,
        never_get,
    ));
    let wrong_setter_owner = v7::leak(SetterMetadata::new::<Other, String>("set", name_type, never_set));
    let wrong_setter_input = v7::leak(SetterMetadata::new::<Record, u32>("set", count_type, never_set));
    let properties = [
        v7::property_metadata("getter_owner", name_type, None, Some(wrong_getter_owner), None),
        v7::property_metadata("getter_output", name_type, None, Some(wrong_getter_output), None),
        v7::property_metadata("setter_owner", name_type, None, None, Some(wrong_setter_owner)),
        v7::property_metadata("setter_input", name_type, None, None, Some(wrong_setter_input)),
    ];
    let errors = model.validate_properties(&properties).expect_err("accessor signatures");
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
