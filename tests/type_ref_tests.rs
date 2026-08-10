// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::TypeRef;

#[test]
fn test_type_ref_retains_the_rust_type_name() {
    assert!(TypeRef::of::<i32>().type_name().ends_with("i32"));
}
