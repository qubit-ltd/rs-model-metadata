// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Runtime coverage for the `Model` attribute macro.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::collections::VecDeque;
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
    #[opaque]
    #[redact(level = "secret")]
    password: String,
}

#[Model(id = "test.attribute.SerdeDefaults")]
struct SerdeDefaults {
    optional: Option<String>,
    values: Vec<String>,
    required: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    explicit_optional: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    explicit_values: Vec<String>,
}

#[Model(id = "test.attribute.CollectionDefaults", no_eq, no_partial_eq, no_hash)]
struct CollectionDefaults {
    linked_list: LinkedList<String>,
    vec_deque: VecDeque<String>,
    hash_map: HashMap<String, String>,
    btree_map: BTreeMap<String, String>,
    hash_set: HashSet<String>,
    btree_set: BTreeSet<String>,
    binary_heap: BinaryHeap<String>,
    empty_array: [String; 0],
    #[keep_serializing]
    kept_values: Vec<String>,
    #[keep_serializing]
    kept_option: Option<String>,
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

/// Verifies optional and empty vector fields are omitted by default.
#[test]
fn test_model_attribute_omits_none_and_empty_vector_fields() {
    let value = SerdeDefaults {
        optional: None,
        values: Vec::new(),
        required: "value".to_owned(),
        explicit_optional: None,
        explicit_values: Vec::new(),
    };

    assert_eq!(
        serde_json::to_string(&value).expect("value should serialize"),
        r#"{"required":"value"}"#
    );
}

/// Verifies omitted vector fields deserialize to their empty default.
#[test]
fn test_model_attribute_defaults_omitted_vector_fields() {
    let value: SerdeDefaults = serde_json::from_str(r#"{"required":"value"}"#).expect("value should deserialize");

    assert_eq!(value.optional, None);
    assert!(value.values.is_empty());
    assert_eq!(value.required, "value");
    assert_eq!(value.explicit_optional, None);
    assert!(value.explicit_values.is_empty());
}

/// Verifies supported empty collections are omitted unless explicitly retained.
#[test]
fn test_model_attribute_omits_empty_collections_and_keeps_marked_values() {
    let value = CollectionDefaults {
        linked_list: LinkedList::new(),
        vec_deque: VecDeque::new(),
        hash_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        hash_set: HashSet::new(),
        btree_set: BTreeSet::new(),
        binary_heap: BinaryHeap::new(),
        empty_array: [],
        kept_values: Vec::new(),
        kept_option: None,
    };

    assert_eq!(
        serde_json::to_string(&value).expect("value should serialize"),
        r#"{"kept_values":[],"kept_option":null}"#
    );
}

/// Verifies omitted supported collections are reconstructed from their
/// defaults.
#[test]
fn test_model_attribute_defaults_omitted_supported_collections() {
    let value: CollectionDefaults =
        serde_json::from_str(r#"{"kept_values":[],"kept_option":null}"#).expect("value should deserialize");

    assert!(value.linked_list.is_empty());
    assert!(value.vec_deque.is_empty());
    assert!(value.hash_map.is_empty());
    assert!(value.btree_map.is_empty());
    assert!(value.hash_set.is_empty());
    assert!(value.btree_set.is_empty());
    assert!(value.binary_heap.is_empty());
    assert!(value.empty_array.is_empty());
    assert!(value.kept_values.is_empty());
    assert_eq!(value.kept_option, None);
}
