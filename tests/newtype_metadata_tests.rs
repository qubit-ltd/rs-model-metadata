// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::FieldMetadata;
use qubit_model_metadata::NewtypeMetadata;
use qubit_model_metadata::TypeRef;

#[test]
fn test_newtype_metadata_exposes_its_inner_field() {
    let field = FieldMetadata::new(0, "value", "i32", TypeRef::of::<i32>(), &[]);
    let metadata = NewtypeMetadata::new(field);

    assert_eq!(metadata.field().name(), "value");
}
