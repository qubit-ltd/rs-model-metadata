// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

use std::collections::HashMap;


#[qubit_model_derive::Model(id = "test.derive.Named", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
struct Named {
    #[sequence(min_items = 1, max_items = 3, unique_items)]
    values: Option<Vec<String>>,
    #[map(min_entries = 1, max_entries = 2)]
    labels: HashMap<String, String>,
    #[sequence(unique_items)]
    fixed_unique_values: [String; 3],
}

#[qubit_model_derive::Model(id = "test.derive.Unit", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
struct Unit;

#[qubit_model_derive::Model(id = "test.derive.Newtype", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
struct Newtype(#[text(max_chars = 8)] String);

#[qubit_model_derive::Enum(id = "test.derive.Fieldless", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
enum Fieldless {
    First,
    Second,
}

#[qubit_model_derive::Enum(id = "test.derive.DataEnum", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
enum DataEnum {
    Unit,
    Tuple(#[text(max_chars = 8)] String),
    Struct {
        #[sequence(max_items = 3)]
        values: Vec<String>,
    },
}

fn main() {}
