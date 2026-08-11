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
    #[field(element(text(repertoire = ascii)))]
    scalar: String,
    #[field(element(decimal(scale = 2)))]
    strings: Vec<String>,
    #[field(element(text(repertoire = ascii)))]
    set: std::collections::HashSet<String>,
}

fn main() {}
