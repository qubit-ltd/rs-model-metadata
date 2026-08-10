// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::ScalarType;

#[test]
fn test_scalar_type_is_comparable() {
    assert_eq!(ScalarType::String, ScalarType::String);
}
