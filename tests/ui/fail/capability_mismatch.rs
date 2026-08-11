// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

#[qubit_model_derive::Model(id = "test.derive.Invalid", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
struct Invalid {
    #[field(text(max_chars = 8))]
    text: i64,
    #[field(sequence(max_items = 8))]
    sequence: String,
    #[field(map(max_entries = 8))]
    map: Vec<String>,
    #[field(time(precision = second))]
    time: String,
    #[field(decimal(scale = 2))]
    decimal: i64,
}

fn main() {}
