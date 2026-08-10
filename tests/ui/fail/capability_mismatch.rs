// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

#[derive(qubit_model_derive::ModelMetadata)]
#[model(id = "test.derive.Invalid")]
struct Invalid {
    #[model(text(max_chars = 8))]
    text: i64,
    #[model(sequence(max_items = 8))]
    sequence: String,
    #[model(map(max_entries = 8))]
    map: Vec<String>,
    #[model(time(precision = second))]
    time: String,
    #[model(decimal(scale = 2))]
    decimal: i64,
}

fn main() {}
