// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::TypeCapabilities;
use qubit_model_metadata::TypeRef;
use qubit_model_metadata::TypeShape;

struct OpaqueType;

#[test]
fn test_type_ref_retains_the_rust_type_name() {
    assert!(TypeRef::of::<i32>().type_name().ends_with("i32"));
}

#[test]
fn test_opaque_type_ref_reports_opaque_shape_and_no_capabilities() {
    let type_ref = TypeRef::opaque::<OpaqueType>();

    assert!(matches!(type_ref.shape(), TypeShape::Opaque));
    assert_eq!(type_ref.capabilities(), TypeCapabilities::NONE);
    assert_eq!(type_ref.element_capabilities(), None);
    assert_eq!(type_ref.strip_optional().type_name(), type_ref.type_name());
    assert!(type_ref.named_metadata().is_none());
}
