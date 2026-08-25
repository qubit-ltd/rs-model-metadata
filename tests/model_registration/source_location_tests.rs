// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Coverage for `SourceLocation` remains in the model-registration integration
//! tests.

use qubit_model_metadata::SourceLocation;

#[test]
fn test_source_location_exposes_parts_and_display() {
    let source = SourceLocation::new("src/model.rs", 12, 7);

    assert_eq!(source.file(), "src/model.rs");
    assert_eq!(source.line(), 12);
    assert_eq!(source.column(), 7);
    assert_eq!(source.to_string(), "src/model.rs:12:7");
}
