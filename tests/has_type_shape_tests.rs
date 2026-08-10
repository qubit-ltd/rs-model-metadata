// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::HasTypeShape;
use qubit_model_metadata::ScalarType;
use qubit_model_metadata::TypeShape;

#[test]
fn test_has_type_shape_exposes_scalar_shape() {
    assert!(matches!(
        <i32 as HasTypeShape>::TYPE_SHAPE,
        TypeShape::Scalar(ScalarType::I32)
    ));
}
