// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Integration tests for stable model identifiers.

use qubit_model_metadata::ModelId;
use qubit_model_metadata::ModelIdError;

const STATIC_MODEL_ID: ModelId = ModelId::new("test.model.StaticModel");

#[test]
fn test_model_id_accepts_nested_modules() {
    let id = ModelId::try_new("qubit.platform.metadata.dictionary.DictEntry")
        .expect("the model ID should be valid");

    assert_eq!(id.as_str(), "qubit.platform.metadata.dictionary.DictEntry");
    assert_eq!(id.type_name(), "DictEntry");
}

#[test]
fn test_model_id_accepts_module_segments_with_underscores() {
    let id = ModelId::try_new("test.platform_core.SomeType")
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
        assert_eq!(ModelId::try_new(value), Err(expected), "{value}");
    }
}

/// Verifies that dynamic callers can validate a model ID before creating a
/// static identifier.
#[test]
fn test_model_id_validates_dynamic_values() {
    assert_eq!(ModelId::validate("test.platform.Model"), Ok(()));
    assert_eq!(
        ModelId::validate("test.platform.model"),
        Err(ModelIdError::InvalidTypeSegment)
    );
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
        let error = ModelId::try_new(value)
            .expect_err("the test value must be invalid");
        assert_eq!(error, expected_error);
        assert_eq!(error.to_string(), expected_message);
    }
}

#[test]
fn test_model_id_new_validates_static_values() {
    assert_eq!(STATIC_MODEL_ID.as_str(), "test.model.StaticModel");
    let panic = std::panic::catch_unwind(|| ModelId::new("test.invalid"));
    assert!(panic.is_err());
}
