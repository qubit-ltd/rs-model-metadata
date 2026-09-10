// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for stable model identifiers.

use qubit_model_metadata::metadata::ModelId;
use qubit_model_metadata::metadata::ModelIdBuf;
use qubit_model_metadata::metadata::ModelIdError;

mod model_id;

const STATIC_MODEL_ID: ModelId = ModelId::new("qubit.platform.iam.User");
const CONST_CHECKED: Result<ModelId, ModelIdError> = ModelId::try_new("Single_segment");

#[test]
fn test_model_id_uses_one_shared_ascii_segment_grammar() {
    for value in [
        "User",
        "qubit.platform.iam.User",
        "Test.mod.Model",
        "mod.Type_2",
        "Single_segment",
    ] {
        let borrowed = ModelId::try_new(value).expect("valid static model ID");
        let owned = ModelIdBuf::parse(value).expect("valid owned model ID");
        assert_eq!(borrowed.as_str(), owned.as_str());
    }
    assert_eq!(STATIC_MODEL_ID.type_name(), "User");
    let single = CONST_CHECKED.expect("const validation");
    assert_eq!(single.as_str(), "Single_segment");
    assert_eq!(single.type_name(), "Single_segment");
}

/// Static IDs can index a registry queried by an unallocated string slice.
#[test]
fn test_static_model_id_borrowed_registry_lookup() {
    let entries = std::collections::HashMap::from([(STATIC_MODEL_ID, "user metadata")]);
    assert_eq!(entries.get("qubit.platform.iam.User"), Some(&"user metadata"));
    assert_eq!(entries.get("qubit.platform.iam.user"), None);
}

/// The infallible constructor rejects the same invalid grammar at runtime.
#[test]
#[should_panic(expected = "invalid model ID")]
fn test_static_model_id_constructor_rejects_invalid_input() {
    let _ = ModelId::new("invalid..Name");
}

#[test]
fn test_model_id_rejects_invalid_segments_consistently() {
    let cases = [
        ("", ModelIdError::Empty),
        ("test..Model", ModelIdError::EmptySegment),
        (".test", ModelIdError::EmptySegment),
        ("test.", ModelIdError::EmptySegment),
        ("1test.Model", ModelIdError::InvalidSegment),
        ("test-model.Model", ModelIdError::InvalidSegment),
        ("tést.Model", ModelIdError::InvalidSegment),
    ];
    for (value, expected) in cases {
        assert_eq!(ModelId::validate(value), Err(expected), "{value}");
        assert_eq!(ModelIdBuf::parse(value), Err(expected), "{value}");
    }
}
