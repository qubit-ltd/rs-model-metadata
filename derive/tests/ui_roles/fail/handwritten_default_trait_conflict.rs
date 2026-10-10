// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result;

use qubit_model_derive::Model;

#[Model]
struct Value;

impl Debug for Value {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter.write_str("Value")
    }
}

fn main() {}
