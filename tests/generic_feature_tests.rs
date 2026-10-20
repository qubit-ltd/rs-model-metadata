// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![cfg(feature = "generic")]

//! Generic metadata specialization cache contracts.

use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_reflect::ReflectionContext;
use qubit_reflect::descriptor::Visibility;

#[Model(id = "generic.CacheFixture")]
struct CacheFixture<T> {
    value: T,
}

#[Enum(id = "generic.VariantFixture")]
enum VariantFixture<T> {
    Named { value: Vec<Option<T>> },
    Pair(T, T),
}

trait AssociatedTypeFixture<Argument> {
    type Item<'a>;
}

impl AssociatedTypeFixture<u8> for u8 {
    type Item<'a> = &'a str;
}

#[Model(id = "generic.AssociatedTypeProjection")]
struct AssociatedTypeProjection<T: AssociatedTypeFixture<u8>> {
    unqualified: T::Item<'static>,
    qualified: <T as AssociatedTypeFixture<u8>>::Item<'static>,
}

#[Model(id = "generic.ConstExpression")]
struct ConstExpressionFixture<const N: usize> {
    values: [u8; N],
}

#[test]
fn generic_static_facts_bind_definition_and_symbolic_fields_per_context() {
    let metadata = TypeMetadata::of::<VariantFixture<String>>();
    let generic = metadata.generic_definition().expect("generic definition");
    let first = ReflectionContext::builder()
        .include_type::<VariantFixture<String>>()
        .build()
        .expect("first context");
    let second = ReflectionContext::builder()
        .include_type::<VariantFixture<String>>()
        .build()
        .expect("second context");
    let first_definition = generic.definition_in(&first).expect("definition in first context");
    let second_definition = generic.definition_in(&second).expect("definition in second context");

    assert_eq!(first_definition.path(), generic.definition_path());
    assert_eq!(second_definition.path(), generic.definition_path());
    assert!(!std::ptr::eq(first_definition.context(), second_definition.context()));
    let symbolic = generic.variants()[0].field("value").expect("symbolic field");
    assert_eq!(symbolic.definition_path(), Some(generic.definition_path()));
    assert!(symbolic.reflect_in(&first).is_none());
    let first_expression = symbolic.type_expression_in(&first).expect("symbolic expression");
    let second_expression = symbolic
        .type_expression_in(&second)
        .expect("symbolic expression in second context");
    assert_eq!(first_expression.display().to_string(), "Vec<Option<T>>");
    assert_eq!(second_expression.display().to_string(), "Vec<Option<T>>");
    let qubit_reflect::descriptor::TypeExpressionKind::Named(vector) = first_expression.kind() else {
        panic!("the generic field should retain its outer named type");
    };
    let qubit_reflect::descriptor::GenericExpressionArgumentKind::Type(option) =
        vector.arguments().get(0).expect("Vec element type").kind()
    else {
        panic!("Vec argument should retain the nested Option type");
    };
    let qubit_reflect::descriptor::TypeExpressionKind::Named(option) = option.kind() else {
        panic!("Option should remain a named type expression");
    };
    let qubit_reflect::descriptor::GenericExpressionArgumentKind::Type(parameter) =
        option.arguments().get(0).expect("Option payload type").kind()
    else {
        panic!("Option argument should retain the generic parameter");
    };
    assert!(matches!(
        parameter.kind(),
        qubit_reflect::descriptor::TypeExpressionKind::ParameterName("T")
    ));
    assert!(core::ptr::eq(parameter.context(), &first));
    let qubit_reflect::descriptor::TypeExpressionKind::Named(second_vector) = second_expression.kind() else {
        panic!("the second context should retain the outer named type");
    };
    let qubit_reflect::descriptor::GenericExpressionArgumentKind::Type(second_option) = second_vector
        .arguments()
        .get(0)
        .expect("second Vec element type")
        .kind()
    else {
        panic!("the second context should retain the nested Option type");
    };
    let qubit_reflect::descriptor::TypeExpressionKind::Named(second_option) = second_option.kind() else {
        panic!("the second context should retain Option as a named expression");
    };
    let qubit_reflect::descriptor::GenericExpressionArgumentKind::Type(second_parameter) = second_option
        .arguments()
        .get(0)
        .expect("second Option payload type")
        .kind()
    else {
        panic!("the second context should retain the generic parameter");
    };
    assert!(matches!(
        second_parameter.kind(),
        qubit_reflect::descriptor::TypeExpressionKind::ParameterName("T")
    ));
    assert!(core::ptr::eq(second_parameter.context(), &second));
    assert!(core::ptr::eq(first_expression.context(), &first));
    assert!(core::ptr::eq(second_expression.context(), &second));
}

#[test]
fn associated_type_arguments_preserve_projection_and_context_structure() {
    use qubit_reflect::descriptor::GenericExpressionArgumentKind;
    use qubit_reflect::descriptor::LifetimeExpression;
    use qubit_reflect::descriptor::TypeExpressionKind;

    let metadata = TypeMetadata::of::<AssociatedTypeProjection<u8>>();
    let generic = metadata.generic_definition().expect("generic definition");
    let first = ReflectionContext::builder()
        .include_type::<AssociatedTypeProjection<u8>>()
        .build()
        .expect("first context");
    let second = ReflectionContext::builder()
        .include_type::<AssociatedTypeProjection<u8>>()
        .build()
        .expect("second context");

    for (field_name, expected_trait_path) in [
        ("unqualified", None),
        ("qualified", Some("AssociatedTypeFixture < u8 >")),
    ] {
        let field = generic
            .fields()
            .iter()
            .find(|field| field.name() == Some(field_name))
            .expect("generic field");
        for context in [&first, &second] {
            let expression = field.type_expression_in(context).expect("associated type expression");
            let TypeExpressionKind::AssociatedType(associated) = expression.kind() else {
                panic!("field type must remain an associated-type projection");
            };
            assert_eq!(associated.name(), "Item");
            assert!(matches!(
                associated.self_type().kind(),
                TypeExpressionKind::ParameterName("T")
            ));
            assert_eq!(associated.arguments().len(), 1);
            assert!(matches!(
                associated
                    .arguments()
                    .get(0)
                    .expect("associated lifetime argument")
                    .kind(),
                GenericExpressionArgumentKind::Lifetime(LifetimeExpression::Static),
            ));
            assert!(core::ptr::eq(associated.arguments().context(), context));
            assert_eq!(associated.trait_().map(|value| value.path()), expected_trait_path);
            if let Some(trait_) = associated.trait_() {
                assert_eq!(trait_.arguments().len(), 1);
                assert!(matches!(
                    trait_.arguments().get(0).expect("trait type argument").kind(),
                    GenericExpressionArgumentKind::Type(argument)
                        if matches!(argument.kind(), TypeExpressionKind::Named(named) if named.path() == "u8")
                ));
                assert!(core::ptr::eq(trait_.arguments().context(), context));
            }
        }
    }
}

#[test]
fn generic_array_length_retains_const_parameter_in_each_context() {
    let metadata = TypeMetadata::of::<ConstExpressionFixture<4>>();
    let generic = metadata.generic_definition().expect("generic definition");
    let first = ReflectionContext::builder()
        .include_type::<ConstExpressionFixture<4>>()
        .build()
        .expect("first context");
    let second = ReflectionContext::builder()
        .include_type::<ConstExpressionFixture<4>>()
        .build()
        .expect("second context");
    let field = generic.fields().first().expect("generic array field");

    for context in [&first, &second] {
        let expression = field.type_expression_in(context).expect("generic array expression");
        let qubit_reflect::descriptor::TypeExpressionKind::Array { element, length } = expression.kind() else {
            panic!("generic field should retain its array structure");
        };
        assert_eq!(length, qubit_reflect::descriptor::ConstExpression::Parameter("N"));
        assert!(core::ptr::eq(element.context(), context));
    }
}

/// Symbolic payloads preserve lookup coordinates without claiming an instance
/// identity.
#[test]
fn test_generic_enum_payloads_separate_definition_and_concrete_field_identity() {
    let concrete = TypeMetadata::of::<VariantFixture<String>>();
    let other = TypeMetadata::of::<VariantFixture<u64>>();
    let definition = concrete.generic_definition().expect("generic definition");
    assert!(core::ptr::eq(
        definition,
        other.generic_definition().expect("same definition")
    ));
    let variants = concrete.as_enum().expect("concrete enum").variants();
    let context = ReflectionContext::builder()
        .include_type::<VariantFixture<String>>()
        .build()
        .expect("reflection context");
    let second_context = ReflectionContext::builder()
        .include_type::<VariantFixture<String>>()
        .build()
        .expect("second reflection context");
    assert_eq!(variants.len(), 2);
    assert_eq!(definition.variants().len(), variants.len());
    for (index, (symbolic, concrete_variant)) in definition.variants().iter().zip(variants).enumerate() {
        assert_eq!(symbolic.index(), index);
        assert_eq!(concrete_variant.index(), index);
        assert!(symbolic.reflect_in(&context).is_none());
        assert_eq!(symbolic.definition_path(), Some(definition.definition_path()));
        assert!(concrete_variant.reflect_in(&context).is_some());
        assert!(core::ptr::eq(
            concrete_variant.reflect_in(&context).unwrap().context(),
            &context
        ));
        assert!(core::ptr::eq(
            concrete_variant.reflect_in(&second_context).unwrap().context(),
            &second_context
        ));
        assert_eq!(concrete_variant.definition_path(), None);
        assert_eq!(symbolic.rust_name(), concrete_variant.rust_name());
        assert_eq!(symbolic.canonical_name(), concrete_variant.canonical_name());
        assert_eq!(symbolic.serialized_name(), concrete_variant.serialized_name());
        assert_eq!(symbolic.deserialized_name(), concrete_variant.deserialized_name());
        assert_eq!(symbolic.is_default(), concrete_variant.is_default());
        assert_eq!(symbolic.fields().len(), concrete_variant.fields().len());
        assert!(symbolic.field_at(symbolic.fields().len()).is_none());
        assert!(concrete_variant.field_at(usize::MAX).is_none());
        assert!(symbolic.field("absent").is_none());
        assert!(concrete_variant.field("absent").is_none());
        for (field_index, symbolic_field) in symbolic.fields().iter().enumerate() {
            assert!(symbolic_field.location().is_none());
            assert!(symbolic_field.reflect_in(&context).is_none());
            assert_eq!(symbolic_field.definition_path(), Some(definition.definition_path()));
            assert_eq!(symbolic_field.visibility(), Visibility::Private);
            let field = concrete_variant.field_at(field_index).expect("concrete payload");
            let reflected_field = field.reflect_in(&context).expect("payload binds to first context");
            let reflected_field_in_second = field
                .reflect_in(&second_context)
                .expect("payload binds to second context");
            assert!(core::ptr::eq(reflected_field.context(), &context));
            assert!(core::ptr::eq(reflected_field_in_second.context(), &second_context));
            assert_eq!(field.definition_path(), None);
            assert_eq!(field.visibility(), Visibility::Private);
            assert!(field.variant_inherited());
            let location = field.location().expect("concrete field identity");
            assert_eq!(location.owner(), concrete.type_id());
            assert_eq!(location.variant(), Some(index));
            assert_eq!(location.index(), field_index);
            let other_location = other.as_enum().expect("other enum").variants()[index]
                .field_at(field_index)
                .expect("other payload")
                .location()
                .expect("other identity");
            assert_ne!(location, other_location);
        }
    }
    assert_eq!(definition.variants()[0].rust_name(), "Named");
    let named = definition.variants()[0].field("value").expect("named symbolic payload");
    assert!(core::ptr::eq(
        named,
        definition.variants()[0].field_at(0).expect("first payload")
    ));
    assert!(core::ptr::eq(
        variants[0].field("value").expect("named concrete payload"),
        variants[0].field_at(0).expect("first concrete payload")
    ));
    assert!(
        definition.variants()[1].field("0").is_none(),
        "tuple indices are not field names"
    );
    assert!(variants[1].field("0").is_none());
}

#[test]
fn test_specialization_cache_is_stable_per_concrete_type() {
    let first = TypeMetadata::try_of::<CacheFixture<u32>>().expect("valid first specialization");
    let second = TypeMetadata::try_of::<CacheFixture<u32>>().expect("valid repeated specialization");
    let different = TypeMetadata::try_of::<CacheFixture<u64>>().expect("valid second specialization");

    assert!(core::ptr::eq(first, second));
    assert!(!core::ptr::eq(first, different));
    assert!(core::ptr::eq(
        first.generic_definition().expect("first generic definition"),
        different.generic_definition().expect("shared generic definition")
    ));
    let definition_field = &first.generic_definition().expect("definition").fields()[0];
    assert!(definition_field.definition_path().is_some());
    assert_eq!(definition_field.visibility(), Visibility::Private);
    assert_eq!(first.fields()[0].visibility(), Visibility::Private);
}

/// Concurrent queries share one metadata allocation for each concrete type.
#[test]
fn test_concurrent_specializations_share_metadata() {
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..16)
            .map(|_| scope.spawn(TypeMetadata::of::<CacheFixture<String>>))
            .collect();
        let expected = TypeMetadata::of::<CacheFixture<String>>();
        for handle in handles {
            assert!(core::ptr::eq(
                handle
                    .join()
                    .expect("metadata query worker should complete successfully"),
                expected
            ));
        }
    });
}
