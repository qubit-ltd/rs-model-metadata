// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use ::model_runtime::TypeMetadata;
use qubit_model_derive::Model;
use qubit_model_derive::ModelProperties;

mod model_runtime {}

#[Model(id = "test.derive.Renamed")]
struct Renamed {
    value: String,
}

#[ModelProperties]
impl Renamed {
    pub fn value(&self) -> &str {
        &self.value
    }
}

fn main() {
    let metadata = TypeMetadata::of::<Renamed>();
    assert_eq!(metadata.fields().len(), 1);
    assert!(metadata.property("value").is_some());
}
