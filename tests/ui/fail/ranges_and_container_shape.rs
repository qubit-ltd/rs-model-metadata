// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

#[qubit_model_derive::Model(id = "test.derive.Invalid", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
struct Invalid {
    #[text(min_chars = 4, max_chars = 3, min_bytes = 8, max_bytes = 7)]
    text: String,
    #[sequence(min_items = 3, max_items = 2)]
    sequence: Vec<String>,
    #[map(min_entries = 2, max_entries = 1)]
    map: HashMap<String, String>,
    #[decimal(precision = 2, scale = 3)]
    decimal: bigdecimal::BigDecimal,
    #[money(precision = 4)]
    money: bigdecimal::BigDecimal,
    #[sequence(unique_items)]
    set: HashSet<String>,
    #[sequence(min_items = 3, max_items = 3)]
    array: [String; 3],
}

fn main() {}
