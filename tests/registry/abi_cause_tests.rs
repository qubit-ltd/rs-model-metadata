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

use qubit_model_metadata::__private::v8;
use qubit_model_metadata::metadata::AbiViolation;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::registry::ModelRegistryErrorKind;
use qubit_model_metadata::registry::ModelRegistryOrigin;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectionContext;

#[Reflect]
struct Invalid {
    value: String,
}

/// Omits the required field overlay to violate the checked ABI.
#[must_use]
fn invalid_metadata() -> &'static TypeMetadata {
    static METADATA: OnceLock<TypeMetadata> = OnceLock::new();
    METADATA.get_or_init(|| {
        v8::GeneratedTypeMetadataBuilder::new::<Invalid>(None, &[], v8::leak(v8::model_role())).finish_unchecked()
    })
}

#[test]
fn test_registry_retains_typed_abi_violation() {
    let context = ReflectionContext::builder()
        .include_type::<Invalid>()
        .build()
        .expect("valid reflection context");
    let source = ModelRegistryOrigin {
        crate_name: "registry-test",
        crate_version: Some("0"),
        module_path: "invalid",
        file: Some("invalid.rs"),
        line: Some(2),
        column: Some(1),
    };
    let error = ModelRegistry::from_static_metadata(&context, &[(invalid_metadata(), &source)])
        .expect_err("missing overlay must fail");
    assert_eq!(error.kind(), ModelRegistryErrorKind::InvalidMetadata);
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
