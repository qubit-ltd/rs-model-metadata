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
use qubit_model_derive::ModelImpl;

#[Model]
struct Profile {
    value: String,
}

#[ModelImpl]
impl Profile {
    fn private(&self) -> &str {
        &self.value
    }

    pub async fn asynchronous(&self) -> String {
        self.value.clone()
    }

    pub fn generic<T>(&self) -> String {
        self.value.clone()
    }

    pub fn set_value(&self, value: String) {
        let _ = value;
    }
}

fn main() {}
