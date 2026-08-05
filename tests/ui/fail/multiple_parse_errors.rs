// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[derive(qubit_model_derive::ModelMetadata)]
#[model(nullable)]
#[model(computed)]
struct Invalid {
    #[model(primary_key(fields(first)))]
    first: String,
    #[model(unknown)]
    second: String,
}

fn main() {}
