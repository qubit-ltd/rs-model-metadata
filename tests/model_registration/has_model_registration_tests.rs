// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for explicit model registration access.

use qubit_model_metadata::HasModelRegistration;
use qubit_model_metadata::HasTypeMetadata;
use qubit_model_metadata::HasTypeShape;
use qubit_model_metadata::ModelId;
use qubit_model_metadata::ModelRegistration;
use qubit_model_metadata::SourceLocation;
use qubit_model_metadata::StructMetadata;
use qubit_model_metadata::TypeCapabilities;
use qubit_model_metadata::TypeIdentity;
use qubit_model_metadata::TypeKind;
use qubit_model_metadata::TypeMetadata;
use qubit_model_metadata::TypeShape;
use qubit_model_metadata::registration_of;

struct Account;

static ACCOUNT_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.registration.Account"),
    TypeIdentity::of::<Account>(),
    TypeKind::Struct(StructMetadata::new(&[])),
    &[],
);
static ACCOUNT_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.registration.Account"),
    &ACCOUNT_METADATA,
    "has_model_registration_tests::Account",
    "has_model_registration_tests",
    SourceLocation::new("tests/model_registration/has_model_registration_tests.rs", 40, 1),
);

impl HasTypeShape for Account {
    const TYPE_SHAPE: TypeShape = TypeShape::Opaque;
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::NONE;
}

impl HasTypeMetadata for Account {
    fn type_metadata() -> &'static TypeMetadata {
        &ACCOUNT_METADATA
    }
}

impl HasModelRegistration for Account {
    fn model_registration() -> &'static ModelRegistration {
        &ACCOUNT_REGISTRATION
    }
}

#[test]
fn test_registration_of_returns_the_type_registration() {
    assert!(core::ptr::eq(registration_of::<Account>(), &ACCOUNT_REGISTRATION,));
}
