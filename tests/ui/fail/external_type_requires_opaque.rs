// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

use std::path::PathBuf;

#[derive(qubit_model_derive::ModelMetadata)]
#[model(id = "test.derive.ModelWithExternalField")]
struct ModelWithExternalField {
    external: PathBuf,
}

fn main() {}
