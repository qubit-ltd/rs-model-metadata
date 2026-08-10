// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for [`FieldPathResolveError`].

use qubit_model_metadata::FieldPathResolveError;

#[test]
fn test_field_path_resolve_error_preserves_missing_field_segment() {
    let error = FieldPathResolveError::FieldNotFound { segment: "name" };

    assert_eq!(
        error,
        FieldPathResolveError::FieldNotFound { segment: "name" }
    );
}
