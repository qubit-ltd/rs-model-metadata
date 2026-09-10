// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow explicit-imports
//! Integration tests for safe erased property access.

use qubit_model_metadata::__private::v7;
use qubit_model_metadata::metadata::BorrowedPropertySlice;
use qubit_model_metadata::metadata::FieldMetadata;
use qubit_model_metadata::metadata::GetterMetadata;
use qubit_model_metadata::metadata::GetterOutputKind;
use qubit_model_metadata::metadata::PropertyAccessError;
use qubit_model_metadata::metadata::PropertySetFailure;
use qubit_model_metadata::metadata::PropertyStorageKind;
use qubit_model_metadata::metadata::PropertyValue;
use qubit_model_metadata::metadata::SetterMetadata;
use qubit_reflect::InvocationOutput;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectedMut;
use qubit_reflect::ReflectedOwned;
use qubit_reflect::ReflectedRef;
use qubit_reflect::TypeDescriptor;

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
    let name = v7::property_metadata("name", name_type, None, Some(name_getter), None);
    let count = v7::property_metadata("count", count_type, None, Some(count_getter), None);
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
        (descriptor.field_at(0).expect("name field")).declaring_type().type_id(),
        descriptor.field_at(0).expect("name field"),
    )));
    let setter = Box::leak(Box::new(SetterMetadata::new::<PropertyFixture, String>(
        "set_name",
        field.type_ref(),
        set_name,
    )));
    let property = v7::property_metadata("name", field.type_ref(), Some(field), None, Some(setter));
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
        (descriptor.field_at(0).expect("name field")).declaring_type().type_id(),
        descriptor.field_at(0).expect("name field"),
    )));
    let getter = Box::leak(Box::new(GetterMetadata::new::<PropertyFixture>(
        "name",
        field.type_ref(),
        GetterOutputKind::Borrowed,
        borrowed_name,
    )));
    let computed = v7::property_metadata("name", field.type_ref(), None, Some(getter), None);
    assert!(matches!(
        computed.get(ReflectedRef::new(&7_u32)),
        Err(PropertyAccessError::TargetTypeMismatch(_)),
    ));

    let fallback = v7::property_metadata("name", field.type_ref(), Some(field), None, None);
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
    let descriptor = TypeDescriptor::of::<PropertyFixture>();
    let name_type = descriptor.field_at(0).expect("name field").field_type();
    let getter = Box::leak(Box::new(GetterMetadata::new::<PropertyFixture>(
        "name",
        name_type,
        GetterOutputKind::Borrowed,
        borrowed_name,
    )));
    let setter = Box::leak(Box::new(SetterMetadata::new::<PropertyFixture, String>(
        "set_name", name_type, set_name,
    )));
    assert_eq!(getter.rust_method_name(), "name");
    assert!(std::ptr::eq(getter.output_type(), name_type));
    assert_eq!(setter.rust_method_name(), "set_name");
    assert!(std::ptr::eq(setter.input_type(), name_type));
    assert!(format!("{getter:?}").contains("Borrowed"));
    assert!(format!("{setter:?}").contains("set_name"));
    let computed = v7::property_metadata("name", name_type, None, Some(getter), None);
    let virtual_property = v7::property_metadata("name", name_type, None, None, Some(setter));
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
        virtual_property.get(ReflectedRef::new(&value)),
        Err(PropertyAccessError::NotReadable)
    ));
    let failure = computed
        .set(
            ReflectedMut::new(&mut value),
            ReflectedOwned::new("replacement-secret".to_owned()),
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
        .set(
            ReflectedMut::new(&mut value),
            replacement.expect("untouched replacement"),
        )
        .expect("retry on writable property");
    assert_eq!(value.name, "replacement-secret");
}

#[test]
fn test_setter_validation_preserves_replacement_before_adapter_execution() {
    fn must_not_run(_: ReflectedMut<'_>, _: ReflectedOwned) -> Result<(), PropertySetFailure> {
        panic!("type validation must precede adapter execution");
    }
    let descriptor = TypeDescriptor::of::<PropertyFixture>();
    let name_type = descriptor.field_at(0).expect("name field").field_type();
    let setter = SetterMetadata::new::<PropertyFixture, String>("set_name", name_type, must_not_run);
    let mut wrong_target = 7_u32;
    let failure = setter
        .set(
            ReflectedMut::new(&mut wrong_target),
            ReflectedOwned::new("retry".to_owned()),
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
        .set(ReflectedMut::new(&mut target), ReflectedOwned::new(42_u32))
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
    fn consume_then_fail(target: ReflectedMut<'_>, value: ReflectedOwned) -> Result<(), PropertySetFailure> {
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
    let descriptor = TypeDescriptor::of::<PropertyFixture>();
    let name_type = descriptor.field_at(0).expect("name field").field_type();
    let setter = SetterMetadata::new::<PropertyFixture, String>("set_name", name_type, consume_then_fail);
    let mut target = PropertyFixture {
        name: "before".to_owned(),
        count: 0,
    };
    let failure = setter
        .set(
            ReflectedMut::new(&mut target),
            ReflectedOwned::new("consumed".to_owned()),
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
    let reflected = descriptor.field_at(0).expect("name field");
    let field = Box::leak(Box::new(FieldMetadata::from_reflect(
        reflected.declaring_type().type_id(),
        reflected,
    )));
    let property = v7::property_metadata("name", field.type_ref(), Some(field), None, None);
    let mut wrong_target = 7_u32;
    let failure = property
        .set(
            ReflectedMut::new(&mut wrong_target),
            ReflectedOwned::new("recovered".to_owned()),
        )
        .expect_err("field target type mismatch");
    assert!(matches!(failure.error(), PropertyAccessError::Field(_)));
    let (_, replacement) = failure.into_parts();
    let mut target = PropertyFixture {
        name: "before".to_owned(),
        count: 0,
    };
    property
        .set(
            ReflectedMut::new(&mut target),
            replacement.expect("field retained replacement"),
        )
        .expect("retry against correct target");
    assert_eq!(target.name, "recovered");
    assert_eq!(wrong_target, 7);
}
