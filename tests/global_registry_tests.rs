// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Integration tests for the process-wide immutable model registry.

use std::sync::Arc;
use std::sync::Barrier;

use qubit_model_metadata::__private::linkme;
use qubit_model_metadata::MODEL_REGISTRATIONS;
use qubit_model_metadata::ModelId;
use qubit_model_metadata::ModelRegistration;
use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::SourceLocation;
use qubit_model_metadata::StructMetadata;
use qubit_model_metadata::TypeIdentity;
use qubit_model_metadata::TypeKind;
use qubit_model_metadata::TypeMetadata;

struct GlobalAccount;

static GLOBAL_ACCOUNT_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.global.GlobalAccount"),
    TypeIdentity::of::<GlobalAccount>(),
    TypeKind::Struct(StructMetadata::new(&[])),
    &[],
);

#[linkme::distributed_slice(MODEL_REGISTRATIONS)]
static GLOBAL_ACCOUNT_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.global.GlobalAccount"),
    &GLOBAL_ACCOUNT_METADATA,
    "global_registry_tests::GlobalAccount",
    module_path!(),
    SourceLocation::new(file!(), line!(), column!()),
);

#[test]
fn test_global_returns_the_same_registry_for_every_thread() {
    let barrier = Arc::new(Barrier::new(8));

    let registrations = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let barrier = Arc::clone(&barrier);
                scope.spawn(move || {
                    barrier.wait();
                    ModelRegistry::try_global().expect("the linked registrations should be valid")
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().expect("the registry thread should not panic"))
            .collect::<Vec<_>>()
    });

    let expected = registrations[0];
    assert!(registrations.iter().all(|actual| core::ptr::eq(*actual, expected)));

    assert!(core::ptr::eq(ModelRegistry::global(), expected));
    assert!(core::ptr::eq(
        expected
            .get("test.global.GlobalAccount")
            .expect("the distributed registration should be visible globally"),
        &GLOBAL_ACCOUNT_METADATA,
    ));
}
