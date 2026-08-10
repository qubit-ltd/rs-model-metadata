// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_derive::ModelMetadata;
use qubit_model_metadata::metadata_of;

#[derive(ModelMetadata)]
#[model(id = "test.derive.Normal")]
struct Normal {
    value: String,
}

fn main() {
    assert_eq!(metadata_of::<Normal>().fields().count(), 1);
}
