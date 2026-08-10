// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[derive(qubit_model_derive::Model)]
#[model(id = "test.derive.InvalidPhone", textual(unexpected))]
struct InvalidPhone {
    number: String,
}

fn main() {}
