// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

use std::collections::HashMap;

use qubit_model_derive::ModelMetadata;

#[derive(ModelMetadata)]
#[model(id = "test.derive.Named")]
struct Named {
    #[model(sequence(min_items = 1, max_items = 3, unique_items))]
    values: Option<Vec<String>>,
    #[model(map(min_entries = 1, max_entries = 2))]
    labels: HashMap<String, String>,
    #[model(sequence(unique_items))]
    fixed_unique_values: [String; 3],
}

#[derive(ModelMetadata)]
#[model(id = "test.derive.Unit")]
struct Unit;

#[derive(ModelMetadata)]
#[model(id = "test.derive.Newtype")]
struct Newtype(#[model(text(max_chars = 8))] String);

#[derive(ModelMetadata)]
#[model(id = "test.derive.Fieldless")]
enum Fieldless {
    First,
    Second,
}

fn main() {}
