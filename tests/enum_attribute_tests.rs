// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for the `Enum` attribute macro.

use qubit_model_derive::Enum;

#[Enum(id = "test.enum_attribute.Status")]
enum Status {
    Draft,
}

#[test]
fn test_enum_attribute_generates_name_conversion() {
    assert_eq!(Status::Draft.name(), "DRAFT");
    assert_eq!(Status::from_name("DRAFT"), Some(Status::Draft));
}
