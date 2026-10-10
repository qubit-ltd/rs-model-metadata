// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use serde::Deserialize;
use serde::Serialize;
use qubit_model_derive::Model;

#[Model(no_serialize)]
struct Value {
    value: String,
}

fn requires_serialize<T: Serialize>() {}
fn requires_deserialize<T: for<'de> Deserialize<'de>>() {}

fn main() {
    requires_deserialize::<Value>();
    requires_serialize::<Value>();
}
