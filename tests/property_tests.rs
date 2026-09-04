// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow explicit-imports
//! Integration tests for safe erased property access.

use qubit_model_metadata::__private::v4;
use qubit_model_metadata::BorrowedPropertySlice;
use qubit_model_metadata::FieldMetadata;
use qubit_model_metadata::GetterMetadata;
use qubit_model_metadata::GetterOutputKind;
use qubit_model_metadata::InvocationOutput;
use qubit_model_metadata::PropertyAccessError;
use qubit_model_metadata::PropertySetFailure;
use qubit_model_metadata::PropertyStorageKind;
use qubit_model_metadata::PropertyValue;
use qubit_model_metadata::Reflect;
use qubit_model_metadata::ReflectedMut;
use qubit_model_metadata::ReflectedOwned;
use qubit_model_metadata::ReflectedRef;
use qubit_model_metadata::SetterMetadata;
use qubit_model_metadata::TypeDescriptor;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct PropertyFixture {
    name: String,
    count: u32,
}

fn borrowed_name<'a>(target: ReflectedRef<'a>) -> Result<PropertyValue<'a>, PropertyAccessError> {
    let target = target
        .downcast::<PropertyFixture>()
        .map_err(|_| PropertyAccessError::user("getter target was not prevalidated"))?;
    Ok(PropertyValue::Borrowed(ReflectedRef::new_str(&target.name)))
}

fn owned_count<'a>(target: ReflectedRef<'a>) -> Result<PropertyValue<'a>, PropertyAccessError> {
    let target = target
        .downcast::<PropertyFixture>()
        .map_err(|_| PropertyAccessError::user("getter target was not prevalidated"))?;
    Ok(PropertyValue::Owned(ReflectedOwned::new(target.count)))
}

fn set_name(target: ReflectedMut<'_>, value: ReflectedOwned) -> Result<(), PropertySetFailure> {
    let target = target.downcast::<PropertyFixture>().map_err(|_| {
        PropertySetFailure::after_execution(PropertyAccessError::user("setter target was not prevalidated"))
    })?;
    let value = value.downcast::<String>().map_err(|value| {
        PropertySetFailure::before_execution(PropertyAccessError::user("setter value was not prevalidated"), value)
    })?;
    target.name = value;
    Ok(())
}

#[test]
fn test_property_supports_borrowed_and_owned_getters() {
    let descriptor = TypeDescriptor::of::<PropertyFixture>();
    let name_type = descriptor.field_at(0).expect("name field").field_type();
    let count_type = descriptor.field_at(1).expect("count field").field_type();
    let name_getter = Box::leak(Box::new(GetterMetadata::new::<PropertyFixture>(
        "name",
        name_type,
        GetterOutputKind::Borrowed,
        borrowed_name,
    )));
    let count_getter = Box::leak(Box::new(GetterMetadata::new::<PropertyFixture>(
        "count",
        count_type,
        GetterOutputKind::Owned,
        owned_count,
    )));
    let name = v4::property_metadata("name", name_type, None, Some(name_getter), None);
    let count = v4::property_metadata("count", count_type, None, Some(count_getter), None);
    let value = PropertyFixture {
        name: "alice".to_owned(),
        count: 7,
    };

    let PropertyValue::Borrowed(name_value) = name.get(ReflectedRef::new(&value)).expect("borrowed getter") else {
        panic!("name getter must borrow");
    };
    assert_eq!(name_value.as_str(), Some("alice"));
    let PropertyValue::Owned(count_value) = count.get(ReflectedRef::new(&value)).expect("owned getter") else {
        panic!("count getter must own");
    };
    assert_eq!(count_value.downcast_ref::<u32>(), Some(&7));
    assert_eq!(name.storage_kind(), PropertyStorageKind::Computed);
}

#[test]
fn test_property_optional_borrow_and_slice_bridge_to_reflection_output() {
    let value = 9_u32;
    let optional = PropertyValue::OptionalBorrowed(Some(ReflectedRef::new(&value))).into_invocation_output();
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
    let field = Box::leak(Box::new(FieldMetadata::from_reflect(
        descriptor.field_at(0).expect("name field"),
    )));
    let setter = Box::leak(Box::new(SetterMetadata::new::<PropertyFixture, String>(
        "set_name",
        field.type_ref(),
        set_name,
    )));
    let property = v4::property_metadata("name", field.type_ref(), Some(field), None, Some(setter));
    let mut value = PropertyFixture {
        name: "before".to_owned(),
        count: 0,
    };

    let PropertyValue::Borrowed(current) = property.get(ReflectedRef::new(&value)).expect("field getter") else {
        panic!("field fallback must borrow");
    };
    assert_eq!(current.downcast_ref::<String>().map(String::as_str), Some("before"));
    property
        .set(ReflectedMut::new(&mut value), ReflectedOwned::new("after".to_owned()))
        .expect("setter");
    assert_eq!(value.name, "after");
    assert_eq!(property.storage_kind(), PropertyStorageKind::FieldBacked);

    let failure = property
        .set(ReflectedMut::new(&mut value), ReflectedOwned::new(42_u32))
        .expect_err("wrong replacement type must fail before execution");
    assert_eq!(
        failure.replacement().and_then(|value| value.downcast_ref::<u32>()),
        Some(&42)
    );
    assert_eq!(value.name, "after");
}

#[test]
fn test_property_rejects_wrong_targets_and_field_fallback_can_write() {
    let descriptor = TypeDescriptor::of::<PropertyFixture>();
    let field = Box::leak(Box::new(FieldMetadata::from_reflect(
        descriptor.field_at(0).expect("name field"),
    )));
    let getter = Box::leak(Box::new(GetterMetadata::new::<PropertyFixture>(
        "name",
        field.type_ref(),
        GetterOutputKind::Borrowed,
        borrowed_name,
    )));
    let computed = v4::property_metadata("name", field.type_ref(), None, Some(getter), None);
    assert!(matches!(
        computed.get(ReflectedRef::new(&7_u32)),
        Err(PropertyAccessError::TargetTypeMismatch(_)),
    ));

    let fallback = v4::property_metadata("name", field.type_ref(), Some(field), None, None);
    let mut value = PropertyFixture {
        name: "before".to_owned(),
        count: 0,
    };
    fallback
        .set(ReflectedMut::new(&mut value), ReflectedOwned::new("field".to_owned()))
        .expect("reflected field setter");
    assert_eq!(value.name, "field");
    assert!(fallback.is_readable());
    assert!(fallback.is_writable());
}
