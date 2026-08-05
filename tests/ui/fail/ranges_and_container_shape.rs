// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

#[derive(qubit_model_derive::ModelMetadata)]
struct Invalid {
    #[model(text(min_chars = 4, max_chars = 3, min_bytes = 8, max_bytes = 7))]
    text: String,
    #[model(sequence(min_items = 3, max_items = 2))]
    sequence: Vec<String>,
    #[model(map(min_entries = 2, max_entries = 1))]
    map: HashMap<String, String>,
    #[model(decimal(precision = 2, scale = 3))]
    decimal: bigdecimal::BigDecimal,
    #[model(money(precision = 4))]
    money: bigdecimal::BigDecimal,
    #[model(sequence(unique_items))]
    set: HashSet<String>,
    #[model(sequence(min_items = 3, max_items = 3))]
    array: [String; 3],
}

fn main() {}
