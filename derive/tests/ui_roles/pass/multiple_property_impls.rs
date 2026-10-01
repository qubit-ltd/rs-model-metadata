// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// The filename is part of a Cargo or trybuild fixture protocol.

//! Verifies that one model accepts multiple property implementation blocks.

use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;

#[Model]
struct Item {
    value: String,
}

#[ModelImpl]
impl Item {
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[ModelImpl]
impl Item {
    pub fn value_length(&self) -> usize {
        self.value.len()
    }
}

fn main() {}
