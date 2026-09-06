// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

use ::model_runtime::ModelRegistry;
use ::model_runtime::TypeMetadata;
use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;

mod model_runtime {}

#[Model(id = "test.derive.Renamed")]
struct Renamed {
    value: String,
}

#[Model(id = "test.derive.RenamedGeneric")]
#[allow(
    dead_code,
    reason = "the registration must not require a concrete monomorph"
)]
struct RenamedGeneric<T> {
    value: T,
}

#[Model]
struct RenamedProperties {
    name: String,
    alias: Option<String>,
    tags: Vec<String>,
}

#[ModelImpl]
impl RenamedProperties {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, value: String) {
        self.name = value;
    }

    pub fn alias(&self) -> Option<&String> {
        self.alias.as_ref()
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }
}

fn main() {
    assert_eq!(TypeMetadata::of::<Renamed>().fields().len(), 1);
    let registry = ModelRegistry::try_global().expect("renamed runtime registrations");
    assert!(registry.generic("test.derive.RenamedGeneric").is_some());
    assert!(registry.metadata("test.derive.RenamedGeneric").is_none());

    let mut properties = RenamedProperties {
        name: "before".to_owned(),
        alias: Some("visible".to_owned()),
        tags: vec!["one".to_owned()],
    };
    assert_eq!(properties.name(), "before");
    properties.set_name("after".to_owned());
    assert_eq!(properties.name(), "after");
    assert_eq!(properties.alias().map(String::as_str), Some("visible"));
    assert_eq!(properties.tags(), &["one"]);
}
