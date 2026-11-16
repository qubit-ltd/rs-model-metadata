// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Crate-private contracts for registry construction errors.

use std::any::TypeId;
use std::error::Error;

use qubit_reflect::Reflect;
use qubit_reflect::ReflectionContext;

use crate::abi_violation::AbiViolation;
use crate::metadata::ModelId;
use crate::reflect_facade::ModelMetadataCapability;
use crate::registry::ModelRegistryError;
use crate::registry::ModelRegistryErrorKind;
use crate::registry::ModelRegistryOrigin;

#[Reflect]
struct Unmodeled;

fn origin() -> ModelRegistryOrigin {
    ModelRegistryOrigin {
        crate_name: "test-crate",
        crate_version: Some("1.2.3"),
        module_path: "tests::registry_error",
        file: Some("registry_error.rs"),
        line: Some(17),
        column: Some(9),
    }
}

#[test]
fn test_capability_failure_retains_its_source() {
    let context = ReflectionContext::builder()
        .include_type::<Unmodeled>()
        .build()
        .expect("valid context");
    let descriptor = context.type_by_id(TypeId::of::<Unmodeled>()).expect("registered type");
    let cause = match descriptor.capability::<ModelMetadataCapability>() {
        Ok(_) => panic!("capability is absent"),
        Err(error) => error,
    };
    let error = ModelRegistryError::capability(cause);

    assert_eq!(error.kind(), ModelRegistryErrorKind::Capability);
    assert!(error.source().is_some());
    assert!(error.to_string().contains("Capability"));
}

#[test]
fn test_abi_failure_retains_identity_sources_and_original_cause() {
    let model_id = ModelId::new("example.InvalidAbi");
    let source = origin();
    let cause = AbiViolation::new("abi.version", "unsupported ABI version");
    let error = ModelRegistryError::invalid_abi(Some(model_id), vec![source.clone()], cause);

    assert_eq!(error.kind(), ModelRegistryErrorKind::InvalidMetadata);
    assert_eq!(error.model_id(), Some(model_id));
    assert_eq!(error.sources(), &[source]);
    assert_eq!(error.abi_cause(), Some(&cause));
    assert_eq!(
        error.source().expect("original ABI cause").to_string(),
        cause.to_string()
    );
    assert!(error.to_string().contains("example.InvalidAbi"));
}

#[test]
fn test_cause_free_failures_keep_their_kind_and_source_coordinates() {
    let model_id = ModelId::new("example.Conflict");
    let source = origin();
    let conflict = ModelRegistryError::conflict(Some(model_id), vec![source.clone()]);
    assert_eq!(conflict.kind(), ModelRegistryErrorKind::RegistrationConflict);
    assert_eq!(conflict.model_id(), Some(model_id));
    assert_eq!(conflict.sources(), std::slice::from_ref(&source));
    assert!(conflict.source().is_none());
    assert!(conflict.abi_cause().is_none());

    let duplicate = ModelRegistryError::duplicate(model_id, vec![source.clone()]);
    assert_eq!(duplicate.kind(), ModelRegistryErrorKind::DuplicateModelId);
    assert_eq!(duplicate.model_id(), Some(model_id));
    assert_eq!(duplicate.sources(), &[source]);
    assert!(duplicate.source().is_none());
    assert!(duplicate.to_string().contains("example.Conflict"));
}
