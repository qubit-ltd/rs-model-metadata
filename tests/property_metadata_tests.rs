// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Runtime coverage for generated model property adapters.

use model_runtime::ModelRegistry;
use model_runtime::ModelResolveErrorKind;
use model_runtime::ModelResolver;
use model_runtime::PropertyStorageKind;
use model_runtime::PropertyValue;
use model_runtime::ReflectedMut;
use model_runtime::ReflectedOwned;
use model_runtime::ReflectedRef;
use model_runtime::ResolveInputs;
use model_runtime::TypeMetadata;
use qubit_codec::ValueCodecRegistry;
use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_validator::ValidatorRegistry;

#[Model]
struct Profile {
    name: String,
    visits: u32,
    alias: Option<String>,
    tags: Vec<String>,
}

#[Model(id = "property.FieldGetterMismatch")]
struct FieldGetterMismatch {
    value: String,
}

#[ModelImpl]
impl FieldGetterMismatch {
    pub fn value(&self) -> u32 {
        self.value.len() as u32
    }
}

#[ModelImpl]
impl Profile {
    pub fn rename_with_prefix(&mut self, prefix: &str) {
        self.name = format!("{prefix}{}", self.name);
    }

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
fn test_model_impl_merges_fields_getters_and_setters() {
    let metadata = TypeMetadata::of::<Profile>();
    assert_eq!(metadata.property_fragments().len(), 10);
    let properties = metadata.try_properties().expect("profile properties must merge");
    let name = properties.property("name").expect("merged name property");
    let visits = properties.property("visits").expect("field property");
    let summary = properties.property("summary").expect("computed property");
    let token = properties.property("token").expect("virtual property");

    assert_eq!(name.storage_kind(), PropertyStorageKind::FieldBacked);
    assert!(name.is_getter());
    assert!(name.is_setter());
    assert_eq!(visits.storage_kind(), PropertyStorageKind::FieldBacked);
    assert_eq!(summary.storage_kind(), PropertyStorageKind::Computed);
    assert_eq!(token.storage_kind(), PropertyStorageKind::Virtual);

    let alias = properties.property("alias").expect("optional borrowed property");
    let tags = properties.property("tags").expect("slice property");
    let mut profile = Profile {
        name: "before".to_owned(),
        visits: 3,
        alias: Some("visible".to_owned()),
        tags: vec!["one".to_owned(), "two".to_owned()],
    };
    profile.rename_with_prefix("");
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

#[test]
fn test_model_impl_reports_field_getter_mismatch_without_panicking() {
    let metadata = TypeMetadata::of::<FieldGetterMismatch>();
    let errors = metadata
        .try_properties()
        .expect_err("field and getter types must be incompatible");

    assert_eq!(errors.errors().len(), 1);
    assert_eq!(errors.errors()[0].property_name(), "value");

    let registry = ModelRegistry::try_global().expect("valid registration index");
    let errors = ModelResolver::new(ResolveInputs {
        models: registry,
        validators: ValidatorRegistry::global(),
        codecs: ValueCodecRegistry::global(),
    })
    .resolve_all()
    .expect_err("invalid local properties must prevent graph publication");
    assert!(
        errors
            .errors()
            .iter()
            .any(|error| error.kind() == ModelResolveErrorKind::InvalidProperties)
    );
}
