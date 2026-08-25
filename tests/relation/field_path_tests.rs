// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for [`FieldPath`].

use qubit_model_metadata::FieldPath;

#[test]
fn test_field_path_preserves_static_segments() {
    let path = FieldPath::new(&["organization", "id"]);

    assert_eq!(path.segments(), &["organization", "id"]);
    assert!(!path.is_empty());
}

#[test]
fn test_field_path_reports_empty_segments() {
    assert!(FieldPath::new(&[]).is_empty());
}
