// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Crate-private property access checks.

use qubit_reflect::Reflect;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::TypeRef;

use crate::field_metadata::FieldMetadata;
use crate::property::BorrowedPropertySlice;
use crate::property::GetterMetadata;
use crate::property::GetterOutputKind;
use crate::property::PropertyAccessError;
use crate::property::PropertyBorrowed;
use crate::property::PropertyMetadata;
use crate::property::PropertyValue;

fn fail_getter<'a>(_: PropertyBorrowed<'a>) -> Result<PropertyValue<'a>, PropertyAccessError> {
    Err(PropertyAccessError::user("getter failed"))
}

#[Reflect]
struct PrivatePropertyFixture {
    secret: String,
}

#[test]
fn unsafe_dynamic_read_bypasses_private_field_visibility() {
    let descriptor = TypeDescriptor::of::<PrivatePropertyFixture>();
    let reflected = descriptor.field_at(0).expect("secret field");
    let field = Box::leak(Box::new(FieldMetadata::from_reflect(
        reflected.declaring_type().type_id(),
        reflected,
    )));
    let property = PropertyMetadata::new("secret", field.type_ref(), Some(field), None, None);
    let value = PrivatePropertyFixture {
        secret: "authorized".to_owned(),
    };

    assert!(matches!(
        property.get_dynamic(PropertyBorrowed::new(&value)),
        Err(PropertyAccessError::Field(_))
    ));

    let PropertyValue::Borrowed(secret) = unsafe { property.get_dynamic_unsafe(PropertyBorrowed::new(&value)) }
        .expect("explicitly authorized private field read")
    else {
        panic!("field fallback must return a borrowed value");
    };
    assert_eq!(secret.downcast_ref::<String>().map(String::as_str), Some("authorized"));
}

#[test]
fn dynamic_read_propagates_getter_errors() {
    let output_type = Box::leak(Box::new(TypeRef::of::<String>()));
    let getter = Box::leak(Box::new(GetterMetadata::new::<PrivatePropertyFixture>(
        "failing_getter",
        output_type,
        GetterOutputKind::Owned,
        fail_getter,
    )));
    let property = PropertyMetadata::new("computed", output_type, None, Some(getter), None);
    let target = PrivatePropertyFixture {
        secret: "value".to_owned(),
    };

    assert!(matches!(
        property.get_dynamic(PropertyBorrowed::new(&target)),
        Err(PropertyAccessError::User("getter failed"))
    ));
}

#[test]
fn borrowed_slice_reports_empty_and_in_bounds_elements() {
    let values = [17_u32, 23_u32];
    let slice = BorrowedPropertySlice::new(&values);

    assert!(!slice.is_empty());
    assert_eq!(slice.len(), 2);
    assert_eq!(
        slice.get(1).and_then(|value| value.downcast_ref::<u32>().copied()),
        Some(23)
    );
    assert!(slice.get(2).is_none());
}

#[test]
fn dynamic_type_mismatch_reports_expected_and_actual_types() {
    let actual = PropertyBorrowed::new(&23_u32);
    let error = PropertyAccessError::value_type_mismatch::<String>(&actual);

    assert!(matches!(error, PropertyAccessError::ValueTypeMismatch(_)));
}
