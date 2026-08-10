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

#[test]
fn test_model_id_errors_describe_each_invalid_reason() {
    let cases = [
        ("", ModelIdError::Empty, "model ID cannot be empty"),
        (
            "test..Model",
            ModelIdError::EmptySegment,
            "model ID cannot contain empty segments",
        ),
        (
            "test-model.Model",
            ModelIdError::InvalidModuleSegment,
            "model ID has an invalid module segment",
        ),
        (
            "test.model",
            ModelIdError::InvalidTypeSegment,
            "model ID has an invalid type segment",
        ),
        (
            "test.mod.Model",
            ModelIdError::KeywordModuleSegment,
            "model ID module segments cannot be Rust keywords",
        ),
    ];

    for (value, expected_error, expected_message) in cases {
        let error = ModelId::try_from_static(value)
            .expect_err("the test value must be invalid");
        assert_eq!(error, expected_error);
        assert_eq!(error.to_string(), expected_message);
    }
}
