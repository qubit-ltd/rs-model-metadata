// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Integration tests for owned stable model identifiers.

use qubit_model_metadata::ModelIdBuf;
use qubit_model_metadata::ModelIdError;

#[test]
fn test_model_id_buf_owns_validated_values() {
    let owned = ModelIdBuf::try_from("test.model.DynamicModel".to_owned())
        .expect("the dynamic model ID should be valid");

    assert_eq!(owned.as_str(), "test.model.DynamicModel");
    assert_eq!(owned.to_string(), "test.model.DynamicModel");
    assert_eq!(AsRef::<str>::as_ref(&owned), owned.as_str());
}

#[test]
fn test_model_id_buf_reuses_static_validation_errors() {
    assert_eq!(
        ModelIdBuf::try_from("test.invalid"),
        Err(ModelIdError::InvalidTypeSegment),
    );
}
