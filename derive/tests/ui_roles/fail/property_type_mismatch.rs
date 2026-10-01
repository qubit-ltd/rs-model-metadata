// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Trybuild compile-fail fixture: `ModelImpl` rejects a setter whose parameter type differs from the property type; this filename is part of the fixture protocol.

use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;

#[Model]
struct Profile {
    value: u32,
}

#[ModelImpl]
impl Profile {
    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn set_value(&mut self, _value: String) {}
}

fn main() {}
