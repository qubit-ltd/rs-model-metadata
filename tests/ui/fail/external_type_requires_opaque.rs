// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

use std::path::PathBuf;

#[qubit_model_derive::Model(id = "test.derive.ModelWithExternalField", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
struct ModelWithExternalField {
    external: PathBuf,
}

fn main() {}
