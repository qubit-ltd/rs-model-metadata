// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use qubit_model_derive::ModelMetadata;

#[derive(ModelMetadata)]
#[model(id = "test.derivé.InvalidUnicode")]
struct InvalidUnicode {
    value: String,
}

fn main() {}
