// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================


type TextAlias = String;

#[qubit_model_derive::Model(id = "test.derive.Valid", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
struct Valid {
    #[unique(ignore_case)]
    #[text(min_chars = 1, max_chars = 8)]
    value: TextAlias,
}

fn main() {}
