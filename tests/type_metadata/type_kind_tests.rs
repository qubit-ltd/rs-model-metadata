// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::StructMetadata;
use qubit_model_metadata::TypeKind;

#[test]
fn test_type_kind_represents_a_struct() {
    assert!(matches!(
        TypeKind::Struct(StructMetadata::new(&[])),
        TypeKind::Struct(_)
    ));
}
