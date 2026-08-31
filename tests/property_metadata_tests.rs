// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Runtime coverage for generated model property adapters.

use model_runtime::PropertyStorageKind;
use model_runtime::PropertyValue;
use model_runtime::ReflectedMut;
use model_runtime::ReflectedOwned;
use model_runtime::ReflectedRef;
use model_runtime::TypeMetadata;
use qubit_model_derive::Model;
use qubit_model_derive::ModelProperties;

#[Model]
struct Profile {
    name: String,
    visits: u32,
    alias: Option<String>,
    tags: Vec<String>,
}

#[ModelProperties]
impl Profile {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, value: String) {
        self.name = value;
    }

    pub fn summary(&self) -> String {
        format!("{}:{}", self.name, self.visits)
    }

    pub fn alias(&self) -> Option<&String> {
        self.alias.as_ref()
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn set_token(&mut self, _value: String) {}
}

#[test]
fn test_model_properties_merge_fields_getters_and_setters() {
    let metadata = TypeMetadata::of::<Profile>();
    let name = metadata.property("name").expect("merged name property");
    let visits = metadata.property("visits").expect("field property");
    let summary = metadata.property("summary").expect("computed property");
    let token = metadata.property("token").expect("virtual property");

    assert_eq!(name.storage_kind(), PropertyStorageKind::FieldBacked);
    assert!(name.is_getter());
    assert!(name.is_setter());
    assert_eq!(visits.storage_kind(), PropertyStorageKind::FieldBacked);
    assert_eq!(summary.storage_kind(), PropertyStorageKind::Computed);
    assert_eq!(token.storage_kind(), PropertyStorageKind::Virtual);

    let alias = metadata.property("alias").expect("optional borrowed property");
    let tags = metadata.property("tags").expect("slice property");
    let mut profile = Profile {
        name: "before".to_owned(),
        visits: 3,
        alias: Some("visible".to_owned()),
        tags: vec!["one".to_owned(), "two".to_owned()],
    };
    let PropertyValue::Borrowed(value) = name.get(ReflectedRef::new(&profile)).expect("borrowed getter") else {
        panic!("name getter must borrow");
    };
    assert_eq!(value.as_str(), Some("before"));
    name.set(ReflectedMut::new(&mut profile), ReflectedOwned::new("after".to_owned()))
        .expect("setter");
    assert_eq!(profile.name, "after");

    let PropertyValue::Owned(value) = summary.get(ReflectedRef::new(&profile)).expect("owned getter") else {
        panic!("summary getter must own");
    };
    assert_eq!(value.downcast_ref::<String>().map(String::as_str), Some("after:3"));

    let PropertyValue::OptionalBorrowed(Some(value)) = alias.get(ReflectedRef::new(&profile)).expect("optional getter")
    else {
        panic!("alias getter must preserve optional borrowing");
    };
    assert_eq!(value.downcast_ref::<String>().map(String::as_str), Some("visible"));

    let PropertyValue::BorrowedSlice(values) = tags.get(ReflectedRef::new(&profile)).expect("slice getter") else {
        panic!("tags getter must preserve slice borrowing");
    };
    assert_eq!(values.len(), 2);
    assert_eq!(
        values
            .get(1)
            .and_then(|value| value.downcast::<String>().ok())
            .map(String::as_str),
        Some("two"),
    );
}
