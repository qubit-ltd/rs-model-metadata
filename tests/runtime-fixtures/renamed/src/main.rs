// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use ::model_runtime::metadata_of;
use qubit_model_derive::ModelMetadata;

mod model_runtime {}

#[derive(ModelMetadata)]
#[model(id = "test.derive.Renamed")]
struct Renamed {
    value: String,
}

fn main() {
    assert_eq!(metadata_of::<Renamed>().fields().count(), 1);
}
