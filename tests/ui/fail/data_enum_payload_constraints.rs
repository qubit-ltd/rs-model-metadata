// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[qubit_model_derive::Enum(id = "test.derive.InvalidPayloadConstraints")]
enum InvalidPayloadConstraints {
    Tuple(#[text(max_chars = 2, min_chars = 3)] String),
    Struct {
        #[sequence(min_items = 3, max_items = 2)]
        values: Vec<String>,
    },
}

fn main() {}
