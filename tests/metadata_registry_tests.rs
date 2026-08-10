// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Integration tests for immutable model registration lookup.

use qubit_model_metadata::ModelId;
use qubit_model_metadata::ModelRegistration;
use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::ModelRegistryError;
use qubit_model_metadata::SourceLocation;
use qubit_model_metadata::StructMetadata;
use qubit_model_metadata::TypeIdentity;
use qubit_model_metadata::TypeKind;
use qubit_model_metadata::TypeMetadata;

struct Account;
struct Organization;
struct DuplicateA;
struct DuplicateZ;

static ACCOUNT_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.metadata.Account"),
    TypeIdentity::of::<Account>(),
    TypeKind::Struct(StructMetadata::new(&[])),
    &[],
);
static ORGANIZATION_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.metadata.Organization"),
    TypeIdentity::of::<Organization>(),
    TypeKind::Struct(StructMetadata::new(&[])),
    &[],
);
static DUPLICATE_A_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.metadata.Duplicate"),
    TypeIdentity::of::<DuplicateA>(),
    TypeKind::Struct(StructMetadata::new(&[])),
    &[],
);
static DUPLICATE_Z_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.metadata.Duplicate"),
    TypeIdentity::of::<DuplicateZ>(),
    TypeKind::Struct(StructMetadata::new(&[])),
    &[],
);
static MISMATCH_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.metadata.Account"),
    TypeIdentity::of::<Account>(),
    TypeKind::Struct(StructMetadata::new(&[])),
    &[],
);

static ACCOUNT_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::from_static("test.metadata.Account"),
    &ACCOUNT_METADATA,
    "test::Account",
    "test::metadata",
    SourceLocation::new("account.rs", 10, 1),
);
static ORGANIZATION_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::from_static("test.metadata.Organization"),
    &ORGANIZATION_METADATA,
    "test::Organization",
    "test::metadata",
    SourceLocation::new("organization.rs", 20, 1),
);
static DUPLICATE_Z_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::from_static("test.metadata.Duplicate"),
    &DUPLICATE_Z_METADATA,
    "z::Model",
    "z::module",
    SourceLocation::new("z.rs", 30, 3),
);
static DUPLICATE_A_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::from_static("test.metadata.Duplicate"),
    &DUPLICATE_A_METADATA,
    "a::Model",
    "a::module",
    SourceLocation::new("a.rs", 20, 2),
);
static MISMATCH_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::from_static("test.metadata.Mismatch"),
    &MISMATCH_METADATA,
    "test::Mismatch",
    "test::metadata",
    SourceLocation::new("mismatch.rs", 40, 4),
);

#[test]
fn test_from_registrations_sorts_and_looks_up_models() {
    let registry = ModelRegistry::from_registrations([
        &ORGANIZATION_REGISTRATION,
        &ACCOUNT_REGISTRATION,
    ])
    .expect("the registrations should be valid and unique");

    let registrations: Vec<_> = registry
        .registrations()
        .map(|registration| registration.id().as_str())
        .collect();
    assert_eq!(
        registrations,
        ["test.metadata.Account", "test.metadata.Organization"]
    );
    assert!(core::ptr::eq(
        registry
            .get("test.metadata.Account")
            .expect("the account metadata should be found"),
        &ACCOUNT_METADATA,
    ));
    assert!(core::ptr::eq(
        registry
            .registration("test.metadata.Organization")
            .expect("the organization registration should be found"),
        &ORGANIZATION_REGISTRATION,
    ));
    assert!(registry.get("test.metadata.Unknown").is_none());
    assert!(registry.registration("test.metadata.Unknown").is_none());
}

#[test]
fn test_from_registrations_reports_duplicate_id_in_stable_order() {
    let error = ModelRegistry::from_registrations([
        &DUPLICATE_Z_REGISTRATION,
        &DUPLICATE_A_REGISTRATION,
    ])
    .expect_err("duplicate model IDs must be rejected");

    match error {
        ModelRegistryError::DuplicateId { id, first, second } => {
            assert_eq!(id.as_str(), "test.metadata.Duplicate");
            assert_eq!(first.rust_type_name(), "a::Model");
            assert_eq!(first.rust_module_path(), "a::module");
            assert_eq!(first.source().file(), "a.rs");
            assert_eq!(first.source().line(), 20);
            assert_eq!(first.source().column(), 2);
            assert_eq!(second.rust_type_name(), "z::Model");
            assert_eq!(second.rust_module_path(), "z::module");
            assert_eq!(second.source().file(), "z.rs");
            assert_eq!(second.source().line(), 30);
            assert_eq!(second.source().column(), 3);
        }
        other => panic!("expected a duplicate-ID error, got {other:?}"),
    }
}

#[test]
fn test_from_registrations_rejects_mismatched_metadata_id() {
    let error = ModelRegistry::from_registrations([&MISMATCH_REGISTRATION])
        .expect_err("registration and metadata IDs must agree");

    match error {
        ModelRegistryError::MetadataIdMismatch {
            registration,
            registration_id,
            metadata_id,
        } => {
            assert!(core::ptr::eq(registration, &MISMATCH_REGISTRATION));
            assert_eq!(registration_id.as_str(), "test.metadata.Mismatch");
            assert_eq!(metadata_id.as_str(), "test.metadata.Account");
        }
        other => panic!("expected a metadata-ID mismatch error, got {other:?}"),
    }
}
