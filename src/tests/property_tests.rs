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

use crate::field_metadata::FieldMetadata;
use crate::property::PropertyBorrowed;
use crate::property::PropertyMetadata;
use crate::property::PropertyValue;

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

    let PropertyValue::Borrowed(secret) = unsafe { property.get_dynamic_unsafe(PropertyBorrowed::new(&value)) }
        .expect("explicitly authorized private field read")
    else {
        panic!("field fallback must return a borrowed value");
    };
    assert_eq!(secret.downcast_ref::<String>().map(String::as_str), Some("authorized"));
}
