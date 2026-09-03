// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Verifies model initialization preserves reflection registration failures.

use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::ModelRegistryErrorKind;
use qubit_model_metadata::Reflect;
use qubit_model_metadata::register_reflected_type;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct DuplicateReflectionSource;

register_reflected_type!(DuplicateReflectionSource);

#[test]
fn test_duplicate_concrete_source_is_reported_by_reflection_registry() {
    let error =
        ModelRegistry::try_global().expect_err("duplicate reflection roots must invalidate model initialization");

    assert_eq!(error.kind(), ModelRegistryErrorKind::ReflectionRegistry);
    assert_eq!(error.sources().len(), 2);
    assert!(error.to_string().contains("reflection registry error"));
}
