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

use qubit_model_metadata::__private::ModelTypeSeal;
use qubit_model_metadata::__private::v1::register_model_capability;
use qubit_model_metadata::__private::v1::type_metadata;
use qubit_model_metadata::HasTypeMetadata;
use qubit_model_metadata::ModelDescriptorExt;
use qubit_model_metadata::Reflect;
use qubit_model_metadata::TypeDescriptor;
use qubit_model_metadata::TypeMetadata;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct Account;

impl ModelTypeSeal for Account {}

impl HasTypeMetadata for Account {
    fn type_metadata() -> &'static TypeMetadata {
        static METADATA: OnceLock<TypeMetadata> = OnceLock::new();
        METADATA.get_or_init(|| type_metadata(TypeDescriptor::of::<Account>()))
    }
}

register_model_capability!(Account, Account::type_metadata);

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
