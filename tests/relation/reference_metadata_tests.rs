// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for [`ReferenceMetadata`].

use qubit_model_metadata::FieldPath;
use qubit_model_metadata::ModelId;
use qubit_model_metadata::ReferenceMetadata;

#[test]
fn test_reference_metadata_preserves_direct_reference_details() {
    let target = ModelId::new("test.metadata.Target");
    let target_field = FieldPath::new(&["id"]);
    let same_as = FieldPath::new(&["account_id"]);
    let reference = ReferenceMetadata::new(target, target_field, true, Some(same_as));

    assert_eq!(reference.target(), target);
    assert_eq!(reference.target_field(), target_field);
    assert!(reference.must_exist());
    assert_eq!(reference.same_as(), Some(same_as));
}
