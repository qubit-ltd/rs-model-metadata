// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::StructMetadata;

#[test]
fn test_struct_metadata_exposes_empty_fields() {
    assert!(StructMetadata::new(&[]).fields().is_empty());
}
