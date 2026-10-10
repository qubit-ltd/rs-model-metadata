// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::hash::Hash;

use qubit_model_derive::Value as ValueRole;

#[ValueRole(no_eq)]
struct Value {
    value: String,
}

fn requires_hash<T: Hash>() {}

fn main() {
    requires_hash::<Value>();
}
