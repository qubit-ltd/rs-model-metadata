// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::TypeIdentity;

#[test]
fn test_type_identity_compares_the_same_rust_type() {
    assert_eq!(TypeIdentity::of::<u64>(), TypeIdentity::of::<u64>());
}
