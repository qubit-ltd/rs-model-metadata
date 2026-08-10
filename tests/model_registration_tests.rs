// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Integration tests for static model registrations.

use qubit_model_metadata::ModelId;
use qubit_model_metadata::ModelRegistration;
use qubit_model_metadata::SourceLocation;
use qubit_model_metadata::StructMetadata;
use qubit_model_metadata::TypeIdentity;
use qubit_model_metadata::TypeKind;
use qubit_model_metadata::TypeMetadata;

struct Account;

static ACCOUNT_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.registration.Account"),
    TypeIdentity::of::<Account>(),
    TypeKind::Struct(StructMetadata::new(&[])),
    &[],
);

#[test]
fn test_model_registration_exposes_declaration_details() {
    let source =
        SourceLocation::new("tests/model_registration_tests.rs", 27, 3);
    let registration = ModelRegistration::new(
        ModelId::from_static("test.registration.Account"),
        &ACCOUNT_METADATA,
        "model_registration_tests::Account",
        "model_registration_tests",
        source,
    );

    assert_eq!(registration.id(), ACCOUNT_METADATA.id());
    assert!(core::ptr::eq(registration.metadata(), &ACCOUNT_METADATA));
    assert_eq!(
        registration.rust_type_name(),
        "model_registration_tests::Account"
    );
    assert_eq!(registration.rust_module_path(), "model_registration_tests");
    assert_eq!(registration.source(), source);
    assert_eq!(registration.to_string(), "model_registration_tests::Account in model_registration_tests at tests/model_registration_tests.rs:27:3");
}
