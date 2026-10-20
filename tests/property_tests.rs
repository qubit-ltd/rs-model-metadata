// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for safe erased property access.

use std::any::TypeId;

use qubit_model_metadata::__private::v8;
use qubit_model_metadata::metadata::BorrowedPropertySlice;
use qubit_model_metadata::metadata::FieldMetadata;
use qubit_model_metadata::metadata::GetterMetadata;
use qubit_model_metadata::metadata::GetterOutputKind;
use qubit_model_metadata::metadata::PropertyAccessError;
use qubit_model_metadata::metadata::PropertyBorrowed;
use qubit_model_metadata::metadata::PropertySetFailure;
use qubit_model_metadata::metadata::PropertyStorageKind;
use qubit_model_metadata::metadata::PropertyTypeMismatch;
use qubit_model_metadata::metadata::PropertyValue;
use qubit_model_metadata::metadata::SetterMetadata;
use qubit_reflect::__private::codegen_v4::expression::ExpressionData;
use qubit_reflect::__private::codegen_v4::expression::TypeExpressionData;
use qubit_reflect::InvocationOutput;
use qubit_reflect::Reflect;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::value::DynamicMut;
use qubit_reflect::value::DynamicOwned;
use qubit_reflect::value::DynamicRef;
use qubit_reflect::value::Local;

static STRING_EXPRESSION: TypeExpressionData = ExpressionData::Named {
    path: "String",
    definition: None,
    arguments: &[],
};
static U32_EXPRESSION: TypeExpressionData = ExpressionData::Named {
    path: "u32",
    definition: None,
    arguments: &[],
};

fn string_type_id() -> TypeId {
    TypeId::of::<String>()
}

fn u32_type_id() -> TypeId {
    TypeId::of::<u32>()
}

#[Reflect]
struct PropertyFixture {
    pub name: String,
    pub count: u32,
}

#[Reflect]
struct PrivatePropertyFixture {
    value: String,
}

fn borrowed_name<'a>(target: DynamicRef<'a, Local>) -> Result<PropertyValue<'a>, PropertyAccessError> {
    let target = target.downcast::<PropertyFixture>().map_err(|failure| {
        let mismatch = failure.into_parts().0;
        PropertyAccessError::TargetTypeMismatch(PropertyTypeMismatch {
            expected: mismatch.expected(),
            actual: mismatch.actual(),
        })
    })?;
    Ok(PropertyValue::Borrowed(DynamicRef::<Local>::new(&target.name)))
}

fn owned_count<'a>(target: DynamicRef<'a, Local>) -> Result<PropertyValue<'a>, PropertyAccessError> {
    let target = target
        .downcast::<PropertyFixture>()
        .map_err(|_| PropertyAccessError::user("getter target was not prevalidated"))?;
    Ok(PropertyValue::Owned(DynamicOwned::<Local>::new(target.count)))
}

fn set_name(target: DynamicMut<'_, Local>, value: DynamicOwned<Local>) -> Result<(), PropertySetFailure> {
    let target = target.downcast::<PropertyFixture>().map_err(|_| {
        PropertySetFailure::after_execution(PropertyAccessError::user("setter target was not prevalidated"))
    })?;
    let value = value.downcast::<String>().map_err(|failure| {
        let (mismatch, recovery) = failure.into_parts();
        PropertySetFailure::before_execution(
            PropertyAccessError::ValueTypeMismatch(PropertyTypeMismatch {
                expected: mismatch.expected(),
                actual: mismatch.actual(),
            }),
            recovery,
        )
    })?;
    target.name = value;
    Ok(())
}

#[test]
fn test_property_adapters_validate_exact_dynamic_value_identity() {
    let name_type = &STRING_EXPRESSION;
    let getter = GetterMetadata::new::<PropertyFixture>(
        "name",
        name_type,
        string_type_id,
        GetterOutputKind::Borrowed,
        borrowed_name,
    );
    let setter = SetterMetadata::new::<PropertyFixture, String>("set_name", name_type, set_name);
    let mut target = PropertyFixture {
        name: "before".to_owned(),
        count: 0,
    };

    let borrowed_type = DynamicRef::<Local>::new(&target).type_id();
    assert_eq!(borrowed_type, TypeId::of::<PropertyFixture>());
    assert!(matches!(
        getter.get_dynamic(DynamicRef::<Local>::new(&target)),
        Ok(PropertyValue::Borrowed(_))
    ));

    let text = DynamicRef::<Local>::new("text");
    assert_eq!(text.type_id(), TypeId::of::<str>());
    let text = DynamicRef::<Local>::new("text");
    assert!(text.as_any().is_none());
    let error = getter
        .get_dynamic(DynamicRef::<Local>::new("text"))
        .err()
        .expect("str target must be rejected");
    let PropertyAccessError::TargetTypeMismatch(mismatch) = error else {
        panic!("exact target mismatch")
    };
    assert_eq!(mismatch.expected, TypeId::of::<PropertyFixture>());
    assert_eq!(mismatch.actual, TypeId::of::<str>());

    let wrong = DynamicMut::<Local>::new(&mut target.count);
    assert_eq!(wrong.type_id(), TypeId::of::<u32>());
    let failure = setter
        .set_dynamic(wrong, DynamicOwned::<Local>::new("retry".to_owned()))
        .expect_err("wrong target");
    let PropertyAccessError::TargetTypeMismatch(mismatch) = failure.error() else {
        panic!("exact target mismatch")
    };
    assert_eq!(mismatch.expected, TypeId::of::<PropertyFixture>());
    assert_eq!(mismatch.actual, TypeId::of::<u32>());
    assert_eq!(
        failure.replacement().and_then(|value| value.downcast_ref::<String>()),
        Some(&"retry".to_owned())
    );

    let replacement = DynamicOwned::<Local>::new(7_u32);
    assert_eq!(replacement.type_id(), TypeId::of::<u32>());
    let failure = setter
        .set_dynamic(DynamicMut::<Local>::new(&mut target), replacement)
        .expect_err("wrong replacement");
    let PropertyAccessError::ValueTypeMismatch(mismatch) = failure.error() else {
        panic!("exact value mismatch")
    };
    assert_eq!(mismatch.expected, TypeId::of::<String>());
    assert_eq!(mismatch.actual, TypeId::of::<u32>());
    assert_eq!(
        failure.replacement().and_then(|value| value.downcast_ref::<u32>()),
        Some(&7)
    );

    setter
        .set_dynamic(
            DynamicMut::<Local>::new(&mut target),
            DynamicOwned::<Local>::new("after".to_owned()),
        )
        .expect("exact write");
    assert_eq!(target.name, "after");
}

#[test]
fn test_property_supports_borrowed_and_owned_getters() {
    let name_type = &STRING_EXPRESSION;
    let count_type = &U32_EXPRESSION;
    let name_getter = Box::leak(Box::new(GetterMetadata::new::<PropertyFixture>(
        "name",
        name_type,
        string_type_id,
        GetterOutputKind::Borrowed,
        borrowed_name,
    )));
    let count_getter = Box::leak(Box::new(GetterMetadata::new::<PropertyFixture>(
        "count",
        count_type,
        u32_type_id,
        GetterOutputKind::Owned,
        owned_count,
    )));
    let name = v8::property_metadata("name", Some(name_type), None, Some(name_getter), None);
    let count = v8::property_metadata("count", Some(count_type), None, Some(count_getter), None);
    let value = PropertyFixture {
        name: "alice".to_owned(),
        count: 7,
    };

    let PropertyValue::Borrowed(name_value) = name
        .get_dynamic(DynamicRef::<Local>::new(&value))
        .expect("borrowed getter")
    else {
        panic!("name getter must borrow");
    };
    assert_eq!(name_value.downcast_ref::<String>().map(String::as_str), Some("alice"));
    let PropertyValue::Owned(count_value) = count
        .get_dynamic(DynamicRef::<Local>::new(&value))
        .expect("owned getter")
    else {
        panic!("count getter must own");
    };
    assert_eq!(count_value.downcast_ref::<u32>(), Some(&7));
    assert_eq!(name.storage_kind(), PropertyStorageKind::Computed);
}

#[test]
fn test_property_optional_borrow_and_slice_bridge_to_reflection_output() {
    let value = 9_u32;
    let optional = PropertyValue::OptionalBorrowed(Some(DynamicRef::<Local>::new(&value))).into_invocation_output();
    let InvocationOutput::OptionalRef { value, origins } = optional else {
        panic!("optional property borrow must remain optional");
    };
    assert_eq!(value.as_ref().and_then(|value| value.downcast_ref::<u32>()), Some(&9),);
    assert_eq!(origins.len(), 1);

    let values = [2_u32, 3_u32];
    let slice = PropertyValue::BorrowedSlice(BorrowedPropertySlice::new(&values)).into_invocation_output();
    let InvocationOutput::RefSlice { values, origins } = slice else {
        panic!("borrowed property slice must remain borrowed");
    };
    assert_eq!(values.len(), 2);
    assert_eq!(values[1].downcast_ref::<u32>(), Some(&3));
    assert_eq!(origins.len(), 1);
}

#[test]
fn test_property_field_fallback_and_setter_recovery_are_safe() {
    let descriptor = TypeDescriptor::of::<PropertyFixture>();
    let reflected = descriptor.fields().get("name").expect("name field");
    let field = Box::leak(Box::new(FieldMetadata::from_reflect(descriptor.type_id(), reflected)));
    let setter = Box::leak(Box::new(SetterMetadata::new::<PropertyFixture, String>(
        "set_name",
        &STRING_EXPRESSION,
        set_name,
    )));
    let property = v8::property_metadata("name", Some(&STRING_EXPRESSION), Some(field), None, Some(setter));
    let mut value = PropertyFixture {
        name: "before".to_owned(),
        count: 0,
    };

    {
        let PropertyValue::Borrowed(current) = property
            .get_dynamic(DynamicRef::<Local>::new(&value))
            .expect("field getter")
        else {
            panic!("field fallback must borrow");
        };
        assert_eq!(current.downcast_ref::<String>().map(String::as_str), Some("before"));
    }
    property
        .set_dynamic(
            DynamicMut::<Local>::new(&mut value),
            DynamicOwned::<Local>::new("after".to_owned()),
        )
        .expect("setter");
    assert_eq!(value.name, "after");
    assert_eq!(property.storage_kind(), PropertyStorageKind::FieldBacked);

    let failure = property
        .set_dynamic(DynamicMut::<Local>::new(&mut value), DynamicOwned::<Local>::new(42_u32))
        .expect_err("wrong replacement type must fail before execution");
    assert_eq!(
        failure.replacement().and_then(|value| value.downcast_ref::<u32>()),
        Some(&42)
    );
    assert_eq!(value.name, "after");
}

#[test]
fn test_safe_property_field_fallback_rejects_private_fields() {
    let descriptor = TypeDescriptor::of::<PrivatePropertyFixture>();
    let reflected = descriptor.fields().get("value").expect("value field");
    let field = Box::leak(Box::new(FieldMetadata::from_reflect(
        reflected.declaring_type().type_id(),
        reflected,
    )));
    let property = v8::property_metadata("value", None, Some(field), None, None);
    let value = PrivatePropertyFixture {
        value: "secret".to_owned(),
    };

    assert!(matches!(property.get(&value), Err(PropertyAccessError::Field(_))));

    // This test explicitly authorizes access to the private field fixture.
    let PropertyValue::Borrowed(secret) =
        unsafe { property.get_dynamic_unsafe(PropertyBorrowed::new(&value)) }.expect("authorized private field read")
    else {
        panic!("field fallback must return a borrowed value");
    };
    assert_eq!(secret.downcast_ref::<String>().map(String::as_str), Some("secret"));
}

#[test]
fn test_property_rejects_wrong_targets_and_field_fallback_can_write() {
    let descriptor = TypeDescriptor::of::<PropertyFixture>();
    let reflected = descriptor.fields().get("name").expect("name field");
    let field = Box::leak(Box::new(FieldMetadata::from_reflect(descriptor.type_id(), reflected)));
    let getter = Box::leak(Box::new(GetterMetadata::new::<PropertyFixture>(
        "name",
        &STRING_EXPRESSION,
        string_type_id,
        GetterOutputKind::Borrowed,
        borrowed_name,
    )));
    let computed = v8::property_metadata("name", Some(&STRING_EXPRESSION), None, Some(getter), None);
    assert!(matches!(
        computed.get_dynamic(DynamicRef::<Local>::new(&7_u32)),
        Err(PropertyAccessError::TargetTypeMismatch(_)),
    ));

    let fallback = v8::property_metadata("name", None, Some(field), None, None);
    let mut value = PropertyFixture {
        name: "before".to_owned(),
        count: 0,
    };
    fallback
        .set_dynamic(
            DynamicMut::<Local>::new(&mut value),
            DynamicOwned::<Local>::new("field".to_owned()),
        )
        .expect("reflected field setter");
    assert_eq!(value.name, "field");
    assert!(fallback.is_readable());
    assert!(fallback.is_writable());
}

#[test]
fn test_borrowed_slice_preserves_element_identity_and_checks_bounds() {
    let values = [String::from("first"), String::from("second")];
    let slice = BorrowedPropertySlice::new(&values);
    assert!(!slice.is_empty());
    assert_eq!(slice.len(), 2);
    let element = slice.get(1).expect("second element");
    assert!(std::ptr::eq(
        element.downcast_ref::<String>().expect("String element"),
        &values[1]
    ));
    assert!(slice.get(2).is_none());
    assert!(slice.get(usize::MAX).is_none());

    let empty = BorrowedPropertySlice::new::<String>(&[]);
    assert!(empty.is_empty());
    assert!(empty.get(0).is_none());
    let InvocationOutput::RefSlice { values, origins } = PropertyValue::BorrowedSlice(empty).into_invocation_output()
    else {
        panic!("empty slice must retain its output kind");
    };
    assert!(values.is_empty());
    assert_eq!(origins.len(), 1);
    let InvocationOutput::OptionalRef { value, .. } = PropertyValue::OptionalBorrowed(None).into_invocation_output()
    else {
        panic!("missing optional borrow must retain its output kind");
    };
    assert!(value.is_none());
}

#[test]
fn test_virtual_and_computed_properties_enforce_access_direction() {
    let name_type = &STRING_EXPRESSION;
    let getter = Box::leak(Box::new(GetterMetadata::new::<PropertyFixture>(
        "name",
        name_type,
        string_type_id,
        GetterOutputKind::Borrowed,
        borrowed_name,
    )));
    let setter = Box::leak(Box::new(SetterMetadata::new::<PropertyFixture, String>(
        "set_name", name_type, set_name,
    )));
    assert_eq!(getter.rust_method_name(), "name");
    assert!(std::ptr::eq(getter.output_type_data(), name_type));
    assert_eq!(setter.rust_method_name(), "set_name");
    assert!(std::ptr::eq(setter.input_type_data(), name_type));
    assert!(format!("{getter:?}").contains("Borrowed"));
    assert!(format!("{setter:?}").contains("set_name"));
    let computed = v8::property_metadata("name", Some(name_type), None, Some(getter), None);
    let virtual_property = v8::property_metadata("name", Some(name_type), None, None, Some(setter));
    assert!(computed.is_readable());
    assert!(!computed.is_writable());
    assert!(computed.is_computed());
    assert!(!virtual_property.is_readable());
    assert!(virtual_property.is_writable());
    assert_eq!(virtual_property.storage_kind(), PropertyStorageKind::Virtual);
    let mut value = PropertyFixture {
        name: "original".to_owned(),
        count: 0,
    };
    assert!(matches!(
        virtual_property.get_dynamic(DynamicRef::<Local>::new(&value)),
        Err(PropertyAccessError::NotReadable)
    ));
    let failure = computed
        .set_dynamic(
            DynamicMut::<Local>::new(&mut value),
            DynamicOwned::<Local>::new("replacement-secret".to_owned()),
        )
        .expect_err("computed property has no write destination");
    assert!(matches!(failure.error(), PropertyAccessError::NotWritable));
    assert_eq!(failure.to_string(), "property is not writable");
    let diagnostic = format!("{failure:?}");
    assert!(diagnostic.contains("has_replacement: true"));
    assert!(!diagnostic.contains("replacement-secret"));
    let (error, replacement) = failure.into_parts();
    assert!(matches!(error, PropertyAccessError::NotWritable));
    virtual_property
        .set_dynamic(
            DynamicMut::<Local>::new(&mut value),
            replacement.expect("untouched replacement"),
        )
        .expect("retry on writable property");
    assert_eq!(value.name, "replacement-secret");
}

#[test]
fn test_setter_validation_preserves_replacement_before_adapter_execution() {
    fn must_not_run(_: DynamicMut<'_, Local>, _: DynamicOwned<Local>) -> Result<(), PropertySetFailure> {
        panic!("type validation must precede adapter execution");
    }
    let name_type = &STRING_EXPRESSION;
    let setter = SetterMetadata::new::<PropertyFixture, String>("set_name", name_type, must_not_run);
    let mut wrong_target = 7_u32;
    let failure = setter
        .set_dynamic(
            DynamicMut::<Local>::new(&mut wrong_target),
            DynamicOwned::<Local>::new("retry".to_owned()),
        )
        .expect_err("wrong target");
    assert!(matches!(failure.error(), PropertyAccessError::TargetTypeMismatch(_)));
    let (_, replacement) = failure.into_parts();
    assert_eq!(
        replacement
            .expect("replacement retained")
            .downcast::<String>()
            .unwrap_or_else(|_| panic!("original type")),
        "retry"
    );
    assert_eq!(wrong_target, 7);

    let mut target = PropertyFixture {
        name: "unchanged".to_owned(),
        count: 0,
    };
    let failure = setter
        .set_dynamic(
            DynamicMut::<Local>::new(&mut target),
            DynamicOwned::<Local>::new(42_u32),
        )
        .expect_err("wrong input");
    assert!(matches!(failure.error(), PropertyAccessError::ValueTypeMismatch(_)));
    let (_, replacement) = failure.into_parts();
    assert_eq!(
        replacement
            .expect("replacement retained")
            .downcast::<u32>()
            .unwrap_or_else(|_| panic!("original type")),
        42
    );
    assert_eq!(target.name, "unchanged");
}

#[test]
fn test_setter_failure_after_execution_does_not_claim_replacement_recovery() {
    fn consume_then_fail(target: DynamicMut<'_, Local>, value: DynamicOwned<Local>) -> Result<(), PropertySetFailure> {
        let target = target
            .downcast::<PropertyFixture>()
            .unwrap_or_else(|_| panic!("validated target"));
        target.name = value
            .downcast::<String>()
            .unwrap_or_else(|_| panic!("validated replacement"));
        Err(PropertySetFailure::after_execution(PropertyAccessError::user(
            "post-write failure",
        )))
    }
    let name_type = &STRING_EXPRESSION;
    let setter = SetterMetadata::new::<PropertyFixture, String>("set_name", name_type, consume_then_fail);
    let mut target = PropertyFixture {
        name: "before".to_owned(),
        count: 0,
    };
    let failure = setter
        .set_dynamic(
            DynamicMut::<Local>::new(&mut target),
            DynamicOwned::<Local>::new("consumed".to_owned()),
        )
        .expect_err("adapter fails after taking replacement");
    assert_eq!(target.name, "consumed");
    assert!(failure.replacement().is_none());
    assert_eq!(failure.to_string(), "property method failed: post-write failure");
    assert!(format!("{failure:?}").contains("has_replacement: false"));
    let (error, replacement) = failure.into_parts();
    assert!(matches!(error, PropertyAccessError::User("post-write failure")));
    assert!(replacement.is_none());
}

#[test]
fn test_field_fallback_failure_retains_replacement_for_retry() {
    let descriptor = TypeDescriptor::of::<PropertyFixture>();
    let reflected = descriptor.fields().get("name").expect("name field");
    let field = Box::leak(Box::new(FieldMetadata::from_reflect(
        reflected.declaring_type().type_id(),
        reflected,
    )));
    let property = v8::property_metadata("name", None, Some(field), None, None);
    let mut wrong_target = 7_u32;
    let failure = property
        .set_dynamic(
            DynamicMut::<Local>::new(&mut wrong_target),
            DynamicOwned::<Local>::new("recovered".to_owned()),
        )
        .expect_err("field target type mismatch");
    assert!(matches!(failure.error(), PropertyAccessError::FieldSet(_)));
    let (_, replacement) = failure.into_parts();
    let mut target = PropertyFixture {
        name: "before".to_owned(),
        count: 0,
    };
    property
        .set_dynamic(
            DynamicMut::<Local>::new(&mut target),
            replacement.expect("field retained replacement"),
        )
        .expect("retry against correct target");
    assert_eq!(target.name, "recovered");
    assert_eq!(wrong_target, 7);
}
