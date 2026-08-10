// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[derive(qubit_model_derive::ModelMetadata)]
#[model(id = "test.derive.Invalid")]
struct Invalid {
    #[model(text(max_chars = 2, min_chars = 3))]
    value: String,
}

fn main() {}
