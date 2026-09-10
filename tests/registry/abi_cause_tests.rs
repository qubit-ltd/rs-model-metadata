// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Registry errors retain checked metadata ABI causes.

use std::error::Error;
use std::sync::OnceLock;

use qubit_model_metadata::__private::ModelMetadataProvider;
use qubit_model_metadata::__private::model_metadata_key;
use qubit_model_metadata::__private::v7;
use qubit_model_metadata::metadata::AbiViolation;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::registry::ModelRegistryErrorKind;
use qubit_reflect::Reflect;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::capability::CapabilityDescriptor;
use qubit_reflect::identity::FragmentIdentity;
use qubit_reflect::registry::RegistrySnapshotBuilder;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct Invalid {
    value: String,
}

/// Omits the required field overlay to violate the checked ABI.
fn invalid_metadata() -> &'static TypeMetadata {
    static METADATA: OnceLock<TypeMetadata> = OnceLock::new();
    METADATA.get_or_init(|| {
        v7::GeneratedTypeMetadataBuilder::new(TypeDescriptor::of::<Invalid>(), None, &[], v7::leak(v7::model_role()))
            .finish_unchecked()
    })
}

#[test]
fn test_registry_retains_typed_abi_violation() {
    let descriptor = TypeDescriptor::of::<Invalid>();
    let source = FragmentIdentity::new("registry-test", "invalid", 1, 1, "type", 1);
    let mut builder = RegistrySnapshotBuilder::new();
    builder.add_type(descriptor, source.clone());
    builder.add_type_capabilities(
        descriptor,
        vec![CapabilityDescriptor::with_adapter(
            model_metadata_key(),
            invalid_metadata as ModelMetadataProvider,
        )],
        FragmentIdentity::new("registry-test", "invalid", 2, 1, "capability", 2),
    );
    let reflection = builder.build().expect("valid reflection snapshot");
    let error = ModelRegistry::from_reflect_registry(&reflection).expect_err("missing overlay must fail");
    assert_eq!(error.kind(), ModelRegistryErrorKind::RegistrationConflict);
    assert_eq!(error.sources(), &[source]);
    let abi = error
        .source()
        .and_then(|source| source.downcast_ref::<AbiViolation>())
        .expect("original ABI cause must remain available");
    assert_eq!(abi.code(), "QMM-ABI-003");
    assert!(!abi.message().is_empty());
    assert_eq!(abi.to_string(), format!("{}: {}", abi.code(), abi.message()));
    assert_eq!(error.abi_cause(), Some(abi));
    assert!(error.to_string().contains(abi.code()));
    assert!(error.to_string().contains(abi.message()));
}
