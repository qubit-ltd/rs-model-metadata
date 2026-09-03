// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow explicit-imports
//! Integration tests for the model reflection facade.

use std::sync::OnceLock;

use bigdecimal::BigDecimal;
use chrono::DateTime;
use chrono::Utc;
use qubit_datatype::DataType;
use qubit_id::Id;
use qubit_model_metadata::__private::ModelTypeSeal;
use qubit_model_metadata::__private::TypeMetadataProvider;
use qubit_model_metadata::__private::v3;
use qubit_model_metadata::__private::v3::register_model_capability;
use qubit_model_metadata::ModelDescriptorExt;
use qubit_model_metadata::Reflect;
use qubit_model_metadata::TypeDescriptor;
use qubit_model_metadata::TypeMetadata;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct Account;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct Impostor;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct ExternalTypeFixture {
    id: Id,
    created_at: DateTime<Utc>,
    amount: BigDecimal,
    request_id: uuid::Uuid,
    data_type: DataType,
}

impl ModelTypeSeal for Account {}

impl TypeMetadataProvider for Account {
    fn __type_metadata() -> &'static TypeMetadata {
        static METADATA: OnceLock<TypeMetadata> = OnceLock::new();
        METADATA.get_or_init(|| {
            let role = v3::leak(v3::model_role());
            v3::GeneratedTypeMetadataBuilder::new(TypeDescriptor::of::<Account>(), None, &[], role).finish::<Account>()
        })
    }
}

impl ModelTypeSeal for Impostor {}

impl TypeMetadataProvider for Impostor {
    fn __type_metadata() -> &'static TypeMetadata {
        Account::__type_metadata()
    }
}

register_model_capability!(Account, Account::__type_metadata);
register_model_capability!(Impostor, Impostor::__type_metadata);

#[test]
fn model_metadata_reuses_the_reflect_descriptor_root() {
    let metadata = TypeMetadata::of::<Account>();
    let descriptor = TypeDescriptor::of::<Account>();

    assert!(std::ptr::eq(metadata.descriptor(), descriptor));
    assert!(std::ptr::eq(
        descriptor.model_metadata().expect("model capability must resolve"),
        metadata
    ));
    assert!(descriptor.is_model_type());
    assert_eq!(metadata.type_id(), descriptor.type_id());
    assert_eq!(metadata.type_name(), descriptor.type_name());
}

#[test]
fn public_metadata_entry_points_reject_cross_type_providers() {
    let direct = std::panic::catch_unwind(TypeMetadata::of::<Impostor>).expect_err("cross-type provider must fail");
    assert!(panic_message(direct).starts_with("QMM-ABI-001:"));

    let capability = std::panic::catch_unwind(|| TypeDescriptor::of::<Impostor>().model_metadata())
        .expect_err("cross-root capability must fail");
    assert!(panic_message(capability).starts_with("QMM-ABI-002:"));
}

#[test]
fn reflect_facade_supports_enabled_ecosystem_and_qubit_types() {
    let descriptor = TypeDescriptor::of::<ExternalTypeFixture>();

    for field in ["id", "created_at", "amount", "request_id", "data_type"] {
        assert!(
            descriptor.field(field).is_some(),
            "missing reflected field {field}"
        );
    }
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&'static str>().map(|value| (*value).to_owned()))
        .expect("ABI panic must contain text")
}
