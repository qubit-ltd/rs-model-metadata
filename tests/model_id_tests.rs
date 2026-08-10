// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Integration tests for stable model identifiers.

use qubit_model_metadata::ModelId;
use qubit_model_metadata::ModelIdError;

#[test]
fn test_model_id_accepts_nested_modules() {
    let id = ModelId::try_from_static(
        "qubit.platform.metadata.dictionary.DictEntry",
    )
    .expect("the model ID should be valid");

    assert_eq!(id.as_str(), "qubit.platform.metadata.dictionary.DictEntry");
    assert_eq!(id.type_name(), "DictEntry");
}

#[test]
fn test_model_id_accepts_module_segments_with_underscores() {
    let id = ModelId::try_from_static("test.platform_core.SomeType")
        .expect("the model ID should be valid");

    assert_eq!(id.type_name(), "SomeType");
}

#[test]
fn test_model_id_rejects_invalid_values() {
    let cases = [
        ("", ModelIdError::Empty),
        ("test..Model", ModelIdError::EmptySegment),
        (".test.Model", ModelIdError::EmptySegment),
        ("test.Model.", ModelIdError::EmptySegment),
        ("test-model.Model", ModelIdError::InvalidModuleSegment),
        ("tést.Model", ModelIdError::InvalidModuleSegment),
        ("1test.Model", ModelIdError::InvalidModuleSegment),
        ("Test.Model", ModelIdError::InvalidModuleSegment),
        ("test.model", ModelIdError::InvalidTypeSegment),
        ("test.mod.Model", ModelIdError::KeywordModuleSegment),
    ];

    for (value, expected) in cases {
        assert_eq!(ModelId::try_from_static(value), Err(expected), "{value}");
    }
}
