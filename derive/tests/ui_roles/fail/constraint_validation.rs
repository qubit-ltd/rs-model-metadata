// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

use qubit_model_derive::Model;

#[Model]
struct InvalidConstraints {
    #[text(min_chars = 1)]
    number: u64,
    #[decimal(scale = 2)]
    decimal: f64,
    #[sequence(min_items = 2, max_items = 1)]
    sequence: Vec<u64>,
    #[sequence(unique_items)]
    set: std::collections::HashSet<u64>,
    #[map(min_entries = 2, max_entries = 1)]
    map: std::collections::HashMap<String, String>,
    #[element(text(max_chars = 4))]
    scalar: String,
}

fn main() {}
