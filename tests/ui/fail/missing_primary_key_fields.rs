// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

#[derive(qubit_model_derive::ModelMetadata)]
#[model(primary_key(generated(id)))]
struct Invalid {
    id: i64,
}

fn main() {}
