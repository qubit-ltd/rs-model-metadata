// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[qubit_model_derive::Model(id = "test.derive.Invalid", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
#[nullable]
#[computed]
struct Invalid {
    #[primary_key(fields(first))]
    first: String,
    #[unknown]
    second: String,
}

fn main() {}
