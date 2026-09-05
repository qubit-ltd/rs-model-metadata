// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for runtime crate path resolution in generated metadata.

use ::model_runtime::TypeMetadata;
use qubit_model_derive::Model;

mod model_runtime {}

#[Model(id = "test.derive.RenamedRuntimeUser")]
#[allow(dead_code)]
struct RenamedRuntimeUser {
    id: i64,
}

#[test]
fn test_model_attribute_uses_absolute_renamed_runtime_path() {
    assert_eq!(TypeMetadata::of::<RenamedRuntimeUser>().fields().len(), 1);
}
