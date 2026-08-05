// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for runtime crate path resolution in generated metadata.

use ::model_runtime::metadata_of;
use qubit_model_derive::ModelMetadata;

mod model_runtime {}

#[derive(ModelMetadata)]
#[allow(dead_code)]
struct RenamedRuntimeUser {
    id: i64,
}

#[test]
fn test_derive_uses_absolute_renamed_runtime_path() {
    assert_eq!(metadata_of::<RenamedRuntimeUser>().fields().count(), 1);
}
