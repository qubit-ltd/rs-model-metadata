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

#[Model(no_redact)]
struct InvalidFields {
    #[unique]
    #[unique(ignore_case = false)]
    name: String,
    #[key_part(order = 1)]
    first: u64,
    #[key_part(order = 1)]
    second: u64,
}

#[Model]
struct NonZeroKeyStart {
    #[key_part(order = 1)]
    first: u64,
}

#[Model]
struct GappedKeyOrder {
    #[key_part(order = 0)]
    first: u64,
    #[key_part(order = 2)]
    third: u64,
}

fn main() {}
