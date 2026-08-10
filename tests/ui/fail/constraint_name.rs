// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[derive(qubit_model_derive::ModelMetadata)]
#[model(
    id = "test.derive.Invalid",
    index(name = "by_value", fields(value)),
    index(name = "by_value", fields(other)),
    key(name = "", fields(value))
)]
struct Invalid {
    #[model(codec = "")]
    value: String,
    other: String,
}

fn main() {}
