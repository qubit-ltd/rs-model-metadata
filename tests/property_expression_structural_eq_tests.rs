// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Property validation compares retained expressions, not their diagnostic
//! text.

use std::any::TypeId;

use qubit_model_derive::Model;
use qubit_model_metadata::__private::codegen_v4::expression::ExpressionData;
use qubit_model_metadata::__private::codegen_v4::expression::GenericExpressionArgument;
use qubit_model_metadata::__private::codegen_v4::expression::LifetimeExpression;
use qubit_model_metadata::__private::codegen_v4::expression::TraitExpressionData;
use qubit_model_metadata::__private::codegen_v4::expression::TraitObjectData;
use qubit_model_metadata::__private::codegen_v4::expression::TypeExpressionData;
use qubit_model_metadata::__private::v8;
use qubit_model_metadata::metadata::GetterMetadata;
use qubit_model_metadata::metadata::GetterOutputKind;
use qubit_model_metadata::metadata::PropertyAccessError;
use qubit_model_metadata::metadata::PropertyBorrowed;
use qubit_model_metadata::metadata::PropertyBuildErrorKind;
use qubit_model_metadata::metadata::PropertyValue;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_reflect::ReflectionContext;

trait ProjectionTrait<'a> {
    type Item;
}

struct Container;

impl<'a> ProjectionTrait<'a> for Container {
    type Item = u8;
}

#[Model]
struct PropertyRecord {
    value: <Container as ProjectionTrait<'static>>::Item,
}

#[Model]
struct AccessorRecord {
    value: u8,
    numbers: Vec<u8>,
}

static U8_TYPE: TypeExpressionData = ExpressionData::Named {
    path: "u8",
    definition: None,
    arguments: &[],
};
static U16_TYPE: TypeExpressionData = ExpressionData::Named {
    path: "u16",
    definition: None,
    arguments: &[],
};
static U8_SLICE: TypeExpressionData = ExpressionData::Slice(&U8_TYPE);
static U16_SLICE: TypeExpressionData = ExpressionData::Slice(&U16_TYPE);
static U8_ITERATOR_ARGUMENTS: [GenericExpressionArgument; 1] =
    [GenericExpressionArgument::AssociatedType {
        name: "Item",
        value: &U8_TYPE,
    }];
static U16_ITERATOR_ARGUMENTS: [GenericExpressionArgument; 1] =
    [GenericExpressionArgument::AssociatedType {
        name: "Item",
        value: &U16_TYPE,
    }];
static U8_ITERATOR: TraitExpressionData =
    TraitExpressionData::new("Iterator", &U8_ITERATOR_ARGUMENTS, &[]);
static U16_ITERATOR: TraitExpressionData =
    TraitExpressionData::new("Iterator", &U16_ITERATOR_ARGUMENTS, &[]);
static U8_ITERATORS: [TraitExpressionData; 1] = [U8_ITERATOR];
static U16_ITERATORS: [TraitExpressionData; 1] = [U16_ITERATOR];
static U8_OBJECT_DATA: TraitObjectData = TraitObjectData::new(&U8_ITERATORS, None);
static U16_OBJECT_DATA: TraitObjectData = TraitObjectData::new(&U16_ITERATORS, None);
static U8_OBJECT: TypeExpressionData = ExpressionData::TraitObject(&U8_OBJECT_DATA);
static U16_OBJECT: TypeExpressionData = ExpressionData::TraitObject(&U16_OBJECT_DATA);
static CONTAINER_TYPE: TypeExpressionData = ExpressionData::Named {
    path: "Container",
    definition: None,
    arguments: &[],
};
static FIELD_TRAIT_ARGUMENTS: [GenericExpressionArgument; 1] =
    [GenericExpressionArgument::Lifetime(
        LifetimeExpression::Static,
    )];
static FIELD_TRAIT_LIFETIMES: [LifetimeExpression; 1] = [LifetimeExpression::Static];
static COLLIDING_TRAIT_LIFETIMES: [LifetimeExpression; 1] = [LifetimeExpression::Named("a")];
static FIELD_TRAIT: TraitExpressionData = TraitExpressionData::new(
    "ProjectionTrait < 'static >",
    &FIELD_TRAIT_ARGUMENTS,
    &FIELD_TRAIT_LIFETIMES,
);
static FIELD_PROJECTION: TypeExpressionData = ExpressionData::AssociatedType {
    self_type: &CONTAINER_TYPE,
    trait_: Some(&FIELD_TRAIT),
    name: "Item",
    arguments: &[],
};
static COLLIDING_FIELD_TRAIT: TraitExpressionData = TraitExpressionData::new(
    "ProjectionTrait < 'static >",
    &FIELD_TRAIT_ARGUMENTS,
    &COLLIDING_TRAIT_LIFETIMES,
);
static COLLIDING_FIELD_PROJECTION: TypeExpressionData = ExpressionData::AssociatedType {
    self_type: &CONTAINER_TYPE,
    trait_: Some(&COLLIDING_FIELD_TRAIT),
    name: "Item",
    arguments: &[],
};

fn context() -> ReflectionContext {
    ReflectionContext::builder()
        .include_type::<PropertyRecord>()
        .include_type::<AccessorRecord>()
        .include_type::<u8>()
        .include_type::<u16>()
        .include_type::<Vec<u8>>()
        .include_type::<[u8]>()
        .include_type::<[u16]>()
        .build()
        .expect("property context")
}

fn never_get<'a>(_: PropertyBorrowed<'a>) -> Result<PropertyValue<'a>, PropertyAccessError> {
    panic!("metadata validation invoked the getter");
}

fn u8_type_id() -> TypeId {
    TypeId::of::<dyn Iterator<Item = u8>>()
}

struct UnregisteredOutput;

fn unregistered_type_id() -> TypeId {
    TypeId::of::<UnregisteredOutput>()
}

fn u8_slice_type_id() -> TypeId {
    TypeId::of::<[u8]>()
}

fn u16_slice_type_id() -> TypeId {
    TypeId::of::<[u16]>()
}

fn output_type_id_u8() -> TypeId {
    TypeId::of::<u8>()
}

fn output_type_id_u16() -> TypeId {
    TypeId::of::<u16>()
}

fn accessor_getter(
    name: &'static str,
    output_expression: &'static TypeExpressionData,
    output_type_id: fn() -> TypeId,
    output_kind: GetterOutputKind,
) -> &'static GetterMetadata {
    v8::leak(GetterMetadata::new::<AccessorRecord>(
        name,
        output_expression,
        output_type_id,
        output_kind,
        never_get,
    ))
}

#[test]
fn test_computed_property_accepts_equal_expression_and_rejects_associated_item_collision() {
    let model = TypeMetadata::of::<PropertyRecord>();
    let context = context();
    let matching_getter = v8::leak(GetterMetadata::new::<PropertyRecord>(
        "computed",
        &U8_OBJECT,
        u8_type_id,
        GetterOutputKind::Borrowed,
        never_get,
    ));
    let matching = v8::property_metadata(
        "computed",
        Some(&U8_OBJECT),
        None,
        Some(matching_getter),
        None,
    );
    model
        .validate_properties_in(&context, &[matching])
        .expect("matching computed property expressions");

    let u8_text =
        qubit_reflect::__private::codegen_v4::expression::type_expression_in(&U8_OBJECT, &context)
            .display()
            .to_string();
    let u16_text =
        qubit_reflect::__private::codegen_v4::expression::type_expression_in(&U16_OBJECT, &context)
            .display()
            .to_string();
    assert_eq!(u8_text, u16_text);

    let colliding_getter = v8::leak(GetterMetadata::new::<PropertyRecord>(
        "computed",
        &U16_OBJECT,
        u8_type_id,
        GetterOutputKind::Borrowed,
        never_get,
    ));
    let colliding = v8::property_metadata(
        "computed",
        Some(&U8_OBJECT),
        None,
        Some(colliding_getter),
        None,
    );
    let errors = model
        .validate_properties_in(&context, &[colliding])
        .expect_err("associated-item collision must be rejected");
    assert_eq!(errors.errors().len(), 1);
    assert_eq!(
        errors.errors()[0].kind(),
        PropertyBuildErrorKind::GetterTypeMismatch
    );
}

#[test]
fn test_field_property_accepts_equal_expression_and_rejects_different_structure() {
    let model = TypeMetadata::of::<PropertyRecord>();
    let context = context();
    let field = model.field("value").expect("value field");
    let field_expression = field.type_expression_data().expect("field expression");
    let expected_projection = qubit_reflect::__private::codegen_v4::expression::type_expression_in(
        &FIELD_PROJECTION,
        &context,
    );
    assert!(
        qubit_reflect::__private::codegen_v4::expression::type_expression_in(
            field_expression,
            &context,
        )
        .structurally_eq(expected_projection)
    );

    let matching = v8::property_metadata("value", Some(field_expression), Some(field), None, None);
    model
        .validate_properties_in(&context, &[matching])
        .expect("matching field expressions");

    let actual_text = qubit_reflect::__private::codegen_v4::expression::type_expression_in(
        field_expression,
        &context,
    )
    .display()
    .to_string();
    let declared_text = qubit_reflect::__private::codegen_v4::expression::type_expression_in(
        &COLLIDING_FIELD_PROJECTION,
        &context,
    )
    .display()
    .to_string();
    assert_eq!(actual_text, declared_text);
    assert!(
        !qubit_reflect::__private::codegen_v4::expression::type_expression_in(
            field_expression,
            &context,
        )
        .structurally_eq(
            qubit_reflect::__private::codegen_v4::expression::type_expression_in(
                &COLLIDING_FIELD_PROJECTION,
                &context,
            )
        )
    );

    let colliding = v8::property_metadata(
        "value",
        Some(&COLLIDING_FIELD_PROJECTION),
        Some(field),
        None,
        None,
    );
    let errors = model
        .validate_properties_in(&context, &[colliding])
        .expect_err("different expression structure must be rejected");
    assert_eq!(errors.errors().len(), 1);
    assert_eq!(
        errors.errors()[0].kind(),
        PropertyBuildErrorKind::FieldTypeMismatch
    );
}

#[test]
fn test_backed_getter_rejects_unregistered_concrete_output_type() {
    let model = TypeMetadata::of::<AccessorRecord>();
    let context = context();
    let field = model.field("value").expect("value field");
    let expression = field.type_expression_data().expect("value expression");
    let getter = accessor_getter(
        "value",
        expression,
        unregistered_type_id,
        GetterOutputKind::Borrowed,
    );
    let property =
        v8::property_metadata("value", Some(expression), Some(field), Some(getter), None);

    let errors = model
        .validate_properties_in(&context, &[property])
        .expect_err("unregistered getter output identity");
    assert_eq!(errors.errors().len(), 1);
    assert_eq!(
        errors.errors()[0].kind(),
        PropertyBuildErrorKind::GetterTypeMismatch
    );
}

#[test]
fn test_sequence_getter_accepts_matching_slice_element_and_rejects_different_element() {
    let model = TypeMetadata::of::<AccessorRecord>();
    let context = context();
    let field = model.field("numbers").expect("numbers field");
    let property_expression = field.type_expression_data().expect("sequence expression");
    let property = |output_expression, output_type_id| {
        let getter = accessor_getter(
            "numbers",
            output_expression,
            output_type_id,
            GetterOutputKind::Borrowed,
        );
        v8::property_metadata(
            "numbers",
            Some(property_expression),
            Some(field),
            Some(getter),
            None,
        )
    };

    model
        .validate_properties_in(&context, &[property(&U8_SLICE, u8_slice_type_id)])
        .expect("Vec<u8> getter can expose [u8]");

    let errors = model
        .validate_properties_in(&context, &[property(&U16_SLICE, u16_slice_type_id)])
        .expect_err("Vec<u8> getter cannot expose [u16]");
    assert_eq!(errors.errors().len(), 1);
    assert_eq!(
        errors.errors()[0].kind(),
        PropertyBuildErrorKind::GetterTypeMismatch
    );
}

#[test]
fn test_borrowed_slice_getter_checks_declared_element_identity() {
    let model = TypeMetadata::of::<AccessorRecord>();
    let context = context();
    let field = model.field("numbers").expect("numbers field");
    let property_expression = field.type_expression_data().expect("sequence expression");
    let property = |element_type_id| {
        let getter = v8::leak(
            GetterMetadata::new::<AccessorRecord>(
                "numbers",
                &U8_SLICE,
                u8_slice_type_id,
                GetterOutputKind::BorrowedSlice,
                never_get,
            )
            .with_output_element_type_id(element_type_id),
        );
        v8::property_metadata(
            "numbers",
            Some(property_expression),
            Some(field),
            Some(getter),
            None,
        )
    };

    model
        .validate_properties_in(&context, &[property(output_type_id_u8)])
        .expect("matching borrowed-slice element");

    let errors = model
        .validate_properties_in(&context, &[property(output_type_id_u16)])
        .expect_err("mismatching borrowed-slice element");
    assert_eq!(errors.errors().len(), 1);
    assert_eq!(
        errors.errors()[0].kind(),
        PropertyBuildErrorKind::GetterTypeMismatch
    );
}

#[test]
fn test_computed_getter_without_property_expression_remains_accepted() {
    let model = TypeMetadata::of::<AccessorRecord>();
    let context = context();
    let getter = accessor_getter(
        "virtual",
        &U8_TYPE,
        output_type_id_u8,
        GetterOutputKind::Borrowed,
    );
    let property = v8::property_metadata("virtual", None, None, Some(getter), None);

    model
        .validate_properties_in(&context, &[property])
        .expect("virtual getter without a retained expression");
}
