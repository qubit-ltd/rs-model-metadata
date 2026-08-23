// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Runtime coverage for the `Model` attribute macro.

use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use model_runtime::metadata_of;
use qubit_model_derive::Enum;
use qubit_model_derive::Model;

#[Enum(id = "test.attribute.Status")]
enum Status {
    Draft,
    InReview,
}

#[Enum(id = "test.attribute.RedactedStatus", redact, no_deserialize)]
enum RedactedStatus {
    InReview,
}

#[Model(id = "test.attribute.User")]
struct User {
    first_name: String,
}

#[Model(id = "test.attribute.Relaxed", no_display, no_eq, no_hash, no_serialize)]
struct Relaxed {
    value: f64,
}

#[Model(id = "test.attribute.DisplayWithoutDebug", no_debug)]
struct DisplayWithoutDebug {
    value: String,
}

#[Model(id = "test.attribute.Credential")]
struct Credential {
    username: String,
    #[field(opaque)]
    #[redact(level = "secret")]
    password: String,
}

/// Verifies default enum traits, canonical names, display text, and metadata.
#[test]
fn test_enum_attribute_supplies_enum_defaults() {
    let status = Status::InReview;
    let copied = status;
    let cloned = status;

    assert_eq!(copied, cloned);
    assert!(Status::Draft < Status::InReview);
    assert_eq!(format!("{status:?}"), "InReview");
    assert_eq!(format!("{status}"), "IN_REVIEW");
    assert_eq!(status.name(), "IN_REVIEW");
    assert_eq!(Status::from_name("IN_REVIEW"), Some(Status::InReview));
    assert_eq!(Status::from_name("missing"), None);
    assert_eq!(
        serde_json::to_string(&status).expect("status should serialize"),
        "\"IN_REVIEW\""
    );
    assert_eq!(metadata_of::<Status>().id().as_str(), "test.attribute.Status");

    let mut hasher = DefaultHasher::new();
    status.hash(&mut hasher);
    let _hash = hasher.finish();
}

/// Verifies redacted serialization keeps enum naming when deserialization is
/// disabled independently.
#[test]
fn test_redacted_enum_without_deserialize_keeps_screaming_snake_case() {
    assert_eq!(
        serde_json::to_string(&RedactedStatus::InReview).expect("redacted status should serialize"),
        "\"IN_REVIEW\"",
    );
}

/// Verifies default struct traits, structured display text, and Serde naming.
#[test]
fn test_model_attribute_supplies_struct_defaults() {
    let user = User {
        first_name: "Ada".to_owned(),
    };
    let cloned = user.clone();

    assert_eq!(cloned, user);
    assert_eq!(format!("{user:?}"), r#"User { first_name: "Ada" }"#);
    assert_eq!(format!("{user}"), r#"User { first_name: "Ada" }"#);
    assert_eq!(
        serde_json::to_string(&user).expect("user should serialize"),
        r#"{"first_name":"Ada"}"#
    );

    let mut hasher = DefaultHasher::new();
    user.hash(&mut hasher);
    let _hash = hasher.finish();
}

/// Verifies disabled capabilities suppress incompatible default derives.
#[test]
fn test_model_attribute_honors_disabled_capabilities() {
    let value = Relaxed { value: 1.5 };
    let cloned = value.clone();

    assert_eq!(cloned.value, 1.5);
    assert_eq!(format!("{value:?}"), "Relaxed { value: 1.5 }");
}

/// Verifies Display remains available when the independent Debug default is
/// off.
#[test]
fn test_model_attribute_supports_display_without_debug() {
    let value = DisplayWithoutDebug {
        value: "safe".to_owned(),
    };

    assert_eq!(format!("{value}"), r#"DisplayWithoutDebug { value: "safe" }"#);
}

/// Verifies field redaction controls formatting and serialization safely.
#[test]
fn test_model_attribute_enables_redaction_for_marked_fields() {
    let value = Credential {
        username: "alice".to_owned(),
        password: "raw-secret".to_owned(),
    };

    assert_eq!(
        format!("{value:?}"),
        r#"Credential { username: "alice", password: "<redacted>" }"#
    );
    assert_eq!(
        format!("{value}"),
        r#"Credential { username: "alice", password: "<redacted>" }"#
    );
    let serialized = serde_json::to_string(&value).expect("credential should serialize");
    assert!(!serialized.contains("raw-secret"));

    let deserialized: Credential = serde_json::from_str(r#"{"username":"alice","password":"input-secret"}"#)
        .expect("credential should deserialize");
    assert_eq!(deserialized.password, "input-secret");
}
