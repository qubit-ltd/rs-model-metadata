// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for the model reflection facade.

use std::any::Any;
use std::sync::OnceLock;

use bigdecimal::BigDecimal;
use chrono::DateTime;
use chrono::Utc;
use qubit_datatype::DataType;
use qubit_id::Id;
use qubit_model_metadata::__private::ModelTypeSeal;
use qubit_model_metadata::__private::TypeMetadataProvider;
use qubit_model_metadata::__private::v8;
use qubit_model_metadata::__private::v8::model_metadata_registration;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;
use uuid::Uuid;

#[Reflect]
struct Account;

#[Reflect]
struct Impostor;

#[Reflect]
struct ExternalTypeFixture {
    id: Id,
    created_at: DateTime<Utc>,
    amount: BigDecimal,
    request_id: Uuid,
    data_type: DataType,
}

impl ModelTypeSeal for Account {}

impl TypeMetadataProvider for Account {
    fn __type_metadata() -> &'static TypeMetadata {
        static METADATA: OnceLock<TypeMetadata> = OnceLock::new();
        METADATA.get_or_init(|| {
            let role = v8::leak(v8::model_role());
            v8::GeneratedTypeMetadataBuilder::new::<Account>(None, &[], role).finish::<Account>()
        })
    }
}

impl ModelTypeSeal for Impostor {}

impl TypeMetadataProvider for Impostor {
    fn __type_metadata() -> &'static TypeMetadata {
        Account::__type_metadata()
    }
}

#[test]
fn test_model_metadata_reuses_the_reflect_descriptor_root() {
    let context = ReflectionContext::builder()
        .include_linked()
        .register(model_metadata_registration::<Account>())
        .build()
        .expect("explicit context must initialize model registrations");
    let metadata = TypeMetadata::of::<Account>();
    let descriptor = metadata.descriptor_in(&context);

    assert_eq!(metadata.descriptor_in(&context).type_id(), descriptor.type_id());
    assert!(std::ptr::eq(descriptor.context(), &context));
    let registry = ModelRegistry::from_context(&context).expect("model registry must initialize");
    assert!(std::ptr::eq(
        registry.context().expect("context-bound registry"),
        &context
    ));
    assert!(std::ptr::eq(
        registry
            .metadata_for(&descriptor)
            .expect("model capability must resolve")
            .expect("model capability must exist"),
        metadata
    ));
    assert_eq!(metadata.type_id(), descriptor.type_id());
    assert_eq!(metadata.type_name(), descriptor.rust_type_name());
}

#[test]
fn test_public_metadata_entry_points_reject_cross_type_providers() {
    let direct = std::panic::catch_unwind(TypeMetadata::of::<Impostor>).expect_err("cross-type provider must fail");
    assert!(panic_message(direct).starts_with("QMM-ABI-001:"));

    let context = ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("explicit context must initialize model registrations");
    let registry = ModelRegistry::from_context(&context).expect("model registry must initialize");
    assert!(
        registry
            .metadata_for(&TypeDescriptor::of_in::<Impostor>(&context))
            .expect("valid absent capability")
            .is_none()
    );
}

#[test]
fn test_reflect_facade_supports_enabled_ecosystem_and_qubit_types() {
    let context = ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("external type reflection context");
    let descriptor = TypeDescriptor::of_in::<ExternalTypeFixture>(&context);

    for field in ["id", "created_at", "amount", "request_id", "data_type"] {
        assert!(descriptor.field(field).is_ok(), "missing reflected field {field}");
    }
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&'static str>().map(|value| (*value).to_owned()))
        .expect("ABI panic must contain text")
}
