// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for the metadata resolver interface.

use qubit_model_metadata::MetadataResolver;
use qubit_model_metadata::ModelId;
use qubit_model_metadata::ModelRegistration;
use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::SourceLocation;
use qubit_model_metadata::StructMetadata;
use qubit_model_metadata::TypeIdentity;
use qubit_model_metadata::TypeKind;
use qubit_model_metadata::TypeMetadata;

struct RegisteredModel;

static REGISTERED_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.resolver.RegisteredModel"),
    TypeIdentity::of::<RegisteredModel>(),
    TypeKind::Struct(StructMetadata::new(&[])),
    &[],
);

static REGISTERED_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.resolver.RegisteredModel"),
    &REGISTERED_METADATA,
    "RegisteredModel",
    "test::resolver",
    SourceLocation::new("metadata_resolver_tests.rs", 1, 1),
);

#[test]
fn test_registry_implements_metadata_resolver() {
    let registry = ModelRegistry::from_registrations(std::iter::empty::<&'static ModelRegistration>())
        .expect("an empty registry should be valid");
    let resolver: &dyn MetadataResolver = &registry;

    assert!(resolver.resolve(TypeIdentity::of::<u32>()).is_none());
}

#[test]
fn test_registry_resolves_registered_type_identity() {
    let registry =
        ModelRegistry::from_registrations([&REGISTERED_REGISTRATION]).expect("the registration should be valid");

    assert!(core::ptr::eq(
        registry
            .resolve(TypeIdentity::of::<RegisteredModel>())
            .expect("the registered type should resolve"),
        &REGISTERED_METADATA,
    ));
}
